use reqwest::{Method, Url};

/// A way to check the status of a service.
#[derive(Debug)]
pub enum StatusCheck {
    /// Do an HTTP request with `method` to `url` and check the status.
    Http { method: Method, url: Url },

    /// Special checking for Pi-Hole.
    PiHole {
        container_name: &'static str,
        port: u16,
        test_domain: &'static str,
    },

    /// Special checking for GoDNS.
    GoDns {
        ip_address_api: Url,
        domain: &'static str,
    },

    /// No checking at all.
    #[allow(dead_code)]
    None,
}
