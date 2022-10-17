use super::status_check::StatusCheck;
use reqwest::{Method, Url};

/// Information needed to check the status of a service and create a
/// `ServiceData` describing it.
#[derive(Debug)]
pub struct Service {
    /// Container name as specified in docker-compose.yaml, e.g. "pi-hole".
    pub container_name: &'static str,

    /// Pretty name for the service, e.g. "Pi-Hole".
    pub name: &'static str,

    /// Link to the service, if applicable.
    pub link: Option<Url>,

    /// How to check the service's status.
    pub status_check: StatusCheck,
}

impl Service {
    pub fn simple_web_service(
        container_name: &'static str,
        name: &'static str,
        url_path: &'static str,
    ) -> Self {
        Self {
            container_name,
            name,
            link: Some(link(container_name, url_path)),
            status_check: StatusCheck::Http {
                method: Method::GET,
                url: link(container_name, url_path),
            },
        }
    }
}

fn link(subdomain: &str, path: &str) -> Url {
    const DOMAIN: &str = "hoppner.se";
    make_url(&format!("https://{subdomain}.{DOMAIN}{path}"))
}

fn make_url(s: &str) -> Url {
    Url::parse(s).unwrap_or_else(|e| panic!("Couldn't parse URL {s:?}: {e}"))
}
