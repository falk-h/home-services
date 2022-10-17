mod service;
mod status_check;
mod status_checker;

use futures::future;
use once_cell::sync::OnceCell;
use reqwest::{Method, Url};
use serde_derive::Serialize;

use service::Service;
use status_check::StatusCheck;
pub use status_checker::StatusChecker;

static SERVICES: OnceCell<Vec<Service>> = OnceCell::new();

/// Data about a service that'll be passed to a handlebars template.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct ServiceData {
    /// Pretty name for the service, e.g. "Pi-Hole".
    name: &'static str,

    /// Link to the service, if applicable.
    link: Option<&'static str>,

    /// Whether or not the service is up and running.
    status: Status,
}

/// Describe's a service's status.
///
/// The contained string
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Status {
    /// WHether or not the service is functional.
    is_up: bool,

    /// A human-readable description of the service's status.
    description: String,
}

impl Ord for ServiceData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(other.name)
    }
}
impl PartialOrd for ServiceData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(other.name)
    }
}

pub async fn check_statuses(checker: &StatusChecker) -> Vec<ServiceData> {
    let services = SERVICES.get_or_init(|| {
        let pi_hole_name = "pi-hole";
        let selftest_url =
            Url::parse("https://hoppner.se/ping").expect("Failed to parse URL for selftest");

        vec![
            Service::simple_web_service("home-assistant", "Home Assistant", "/"),
            Service::simple_web_service("nextcloud", "Nextcloud", "/"),
            Service {
                container_name: "website",
                name: "Selftest",
                link: Some(selftest_url.clone()),
                status_check: StatusCheck::Http {
                    method: Method::GET,
                    url: selftest_url,
                },
            },
            Service {
                container_name: "godns",
                name: "GoDNS",
                link: None,
                status_check: StatusCheck::GoDns {
                    ip_address_api: Url::parse("https://api.ipify.org/")
                        .expect("Failed to parse IP address API URL"),
                    domain: "hoppner.se",
                },
            },
            Service {
                container_name: pi_hole_name,
                name: "Pi-Hole",
                link: None,
                status_check: StatusCheck::PiHole {
                    container_name: pi_hole_name,
                    port: 53,
                    test_domain: "google.com",
                },
            },
        ]
    });

    let futures = services.iter().map(|s| async {
        let status = checker.check(&s.status_check).await;
        let name = s.name;
        let link = s.link.as_ref().map(|url| url.as_str());

        ServiceData { name, link, status }
    });

    let mut statuses = future::join_all(futures).await;
    statuses.sort_unstable();
    statuses
}
