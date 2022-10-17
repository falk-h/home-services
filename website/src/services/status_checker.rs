use std::{
    net::{IpAddr, SocketAddr},
    sync::RwLock,
    time::{Duration, Instant},
};

use futures::future;
use reqwest::{Client, Method, Request, Url};
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    AsyncResolver,
};

use crate::Resolver;

use super::{status_check::StatusCheck, Status};

/// Struct that owns persistent data that's needed to check services' statuses,
/// e.g. the HTTP client.
#[derive(Debug)]
pub struct StatusChecker {
    http_client: Client,
    resolver: Resolver,
    ip_info: RwLock<Option<IpInfo>>,
}

#[derive(Debug, Clone)]
struct IpInfo {
    addr: IpAddr,
    looked_up: Instant,
}

impl StatusChecker {
    pub async fn new() -> Self {
        let resolver =
            AsyncResolver::tokio_from_system_conf().expect("Failed to create DNS client");

        let swag = resolver
            .lookup_ip("swag")
            .await
            .expect("Failed to look up SWAG IP")
            .iter()
            .next()
            .expect("Didn't get any IP for SWAG");
        let swag = SocketAddr::new(swag, 53);

        let http_client = Client::builder()
            .connect_timeout(Duration::from_millis(1000))
            .timeout(Duration::from_millis(2000))
            .resolve_to_addrs("hoppner.se", &[swag])
            .build()
            .unwrap_or_else(|e| panic!("Couldn't create HTTP Client: {e}"));

        Self {
            http_client,
            resolver,
            ip_info: RwLock::new(None),
        }
    }

    pub async fn check(&self, check: &StatusCheck) -> Status {
        let res = match check {
            StatusCheck::Http { method, url } => {
                self.http_status_check(method.clone(), url.clone()).await
            }
            StatusCheck::None => Ok("Not checked, assumed to be up.".to_owned()),
            StatusCheck::PiHole {
                container_name,
                port,
                test_domain,
            } => {
                self.pi_hole_status_check(container_name, *port, test_domain)
                    .await
            }
            StatusCheck::GoDns {
                ip_address_api,
                domain,
            } => {
                self.godns_status_check(ip_address_api.clone(), domain)
                    .await
            }
        };

        let is_up = res.is_ok();
        let description = match res {
            Ok(msg) => msg,
            Err(e) => e.to_string(),
        };

        Status { is_up, description }
    }

    async fn pi_hole_status_check(
        &self,
        container_name: &str,
        port: u16,
        test_domain: &str,
    ) -> anyhow::Result<String> {
        let pi_hole = self
            .resolver
            .lookup_ip(container_name)
            .await?
            .iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Didn't find any IP for {container_name}"))?;
        tracing::debug!("Got Pi-Hole IP {pi_hole}");

        let sockaddr = SocketAddr::new(pi_hole, port);
        let ns_config = NameServerConfig::new(sockaddr, Protocol::Udp);

        let mut config = ResolverConfig::new();
        config.add_name_server(ns_config);

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(1);

        let pi_hole_resolver = Resolver::tokio(config, opts)?;

        let result = pi_hole_resolver.lookup_ip(test_domain).await.map_err(|e| {
            anyhow::anyhow!("Failed to look up {test_domain} via {container_name}: {e}")
        })?;

        tracing::debug!(
            "Got records for {test_domain}: {:?}",
            result.as_lookup().records(),
        );

        Ok(format!(
            "Successfully looked up {test_domain} via {container_name} at {sockaddr}"
        ))
    }

    async fn godns_status_check(
        &self,
        ip_address_api: Url,
        domain: &str,
    ) -> anyhow::Result<String> {
        let domain_ip = async {
            let ip = self
                .resolver
                .lookup_ip(domain)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to look up the IP for {domain}: {e}"))?
                .iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("Didn't find any IP address for {domain}"))?;

            Ok::<_, anyhow::Error>(ip)
        };

        let own_ip = self.own_ip_fallback(ip_address_api.clone());

        let (domain_ip, own_ip) = future::join(domain_ip, own_ip).await;
        let domain_ip = domain_ip?;
        let own_ip = own_ip?;

        if domain_ip == own_ip {
            Ok(format!(
                "{domain} points at the same IP as returned by {ip_address_api}",
            ))
        } else {
            anyhow::bail!("{domain} points at {domain_ip}, and not at {own_ip}")
        }
    }

    async fn own_ip_fallback(&self, ip_address_api: Url) -> anyhow::Result<IpAddr> {
        self.own_ip(ip_address_api)
            .await
            .map(Ok)
            .unwrap_or_else(|e| {
                let ip_info_guard = self.ip_info.read().unwrap();
                if let Some(ip_info) = &*ip_info_guard {
                    let addr = ip_info.addr;
                    drop(ip_info_guard);

                    tracing::error!("Failed to get own IP: {e}. Falling back to {addr}");
                    Ok(addr)
                } else {
                    anyhow::bail!(
                        "Failed to get own IP and don't have any old IP to fall back on: {e}",
                    )
                }
            })
    }

    async fn own_ip(&self, ip_address_api: Url) -> anyhow::Result<IpAddr> {
        const IP_ADDR_CACHE_DURATION: Duration = Duration::from_secs(10 * 60);

        let now = Instant::now();
        {
            let ip_info_guard = self.ip_info.read().unwrap();
            if let Some(ip_info) = &*ip_info_guard {
                let age = now.duration_since(ip_info.looked_up);

                if age < IP_ADDR_CACHE_DURATION {
                    let addr = ip_info.addr;

                    drop(ip_info_guard);
                    tracing::debug!(
                        "Reusing IP adress {addr} because it's only {} old",
                        humantime::format_duration(age),
                    );
                    return Ok(addr);
                }
            }
        }

        let body = self
            .http_client
            .get(ip_address_api.clone())
            .send()
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to look up own IP address from {ip_address_api}: {e}")
            })?
            .bytes()
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to read bytes while looking up IP addres from {ip_address_api}: {e}"
                )
            })?;

        let response = std::str::from_utf8(&body).map_err(|e| {
            anyhow::anyhow!("Response from {ip_address_api} contained invalid UTF-8: {e}")
        })?;

        let addr = response.parse().map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse response {response:?} from {ip_address_api} to an IP address: {e}"
            )
        })?;

        let looked_up = Instant::now();
        *self.ip_info.write().unwrap() = Some(IpInfo { addr, looked_up });

        Ok(addr)
    }

    async fn http_status_check(&self, method: Method, url: Url) -> anyhow::Result<String> {
        let request_str = format!("{method} {url}");

        let request = Request::new(method, url);

        match self.http_client.execute(request).await {
            Ok(response) => {
                let status = response.status();
                let msg = format!("Got {status} from {request_str}");

                if status.is_success() {
                    Ok(msg)
                } else {
                    anyhow::bail!(msg)
                }
            }
            Err(err) => anyhow::bail!("{request_str} failed: {err}"),
        }
    }
}
