use std::sync::Arc;

use quinn::Endpoint;

use crate::NoServerVerification;

pub fn create_config() -> quinn::ClientConfig {
    // Exactly the same as `with_safe_defaults()` but with TLS 1.2 disabled (Quic requires TLS 1.3)
    let mut crypto = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])
        .unwrap()
        .with_custom_certificate_verifier(Arc::new(NoServerVerification))
        .with_no_client_auth();
    crypto.enable_early_data = true;

    quinn::ClientConfig::new(Arc::new(crypto))
}

pub fn create_endpoint() -> std::io::Result<Endpoint> {
    let mut endpoint = crate::client(crate::IPV6_WILDCARD)?;
    let config = create_config();
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}
