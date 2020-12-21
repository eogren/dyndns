use std::net::Ipv4Addr;

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {
       foreign_links {
         ConnectionError(::reqwest::Error);
       }

    }
}

use errors::*;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum PublicAddress {
    IP4Address(String),
}

/// Retrieve the public ip address for this machine using https://ifconfig.co.
pub async fn get_public_ip_address() -> Result<PublicAddress> {
    internal_get_address("https://ifconfig.co/".into()).await
}

/// Retrieve the public ip address for this machine using a different IP service.
/// The service used must simply return the ipv4 address in text format with a 200 OK response.
pub async fn get_public_ip_address_with_override(host: &str) -> Result<PublicAddress> {
    internal_get_address(host).await
}

async fn internal_get_address(host: &str) -> Result<PublicAddress> {
    let resp = reqwest::get(host).await?;
    match resp.error_for_status() {
        Ok(res) => {
            let text = res.text().await?;
            match text.parse::<Ipv4Addr>() {
                Ok(_) => Ok(PublicAddress::IP4Address(text)),
                Err(e) => Err(format!(
                    "Failed parsing {} as ipv4 address: {}",
                    &text,
                    e.to_string()
                )
                .into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use httptest::{matchers::request, responders::status_code, Expectation, Server};

    use crate::{get_public_ip_address_with_override, PublicAddress};

    #[tokio::test]
    async fn test_successful_response() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/"))
                .respond_with(status_code(200).body("1.2.3.4")),
        );

        assert_eq!(
            get_public_ip_address_with_override(&server.url("/").to_string())
                .await
                .unwrap(),
            PublicAddress::IP4Address("1.2.3.4".to_string())
        );
    }

    #[tokio::test]
    async fn test_server_error() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/")).respond_with(status_code(500)),
        );

        assert!(
            get_public_ip_address_with_override(&server.url("/").to_string())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_server_unparseableresponse() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("GET", "/"))
                .respond_with(status_code(200).body("whargarbl")),
        );

        assert!(
            get_public_ip_address_with_override(&server.url("/").to_string())
                .await
                .is_err()
        );
    }
}
