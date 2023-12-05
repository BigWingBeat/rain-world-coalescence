use quinn::Endpoint;
use rcgen::RcgenError;
use rustls::{Certificate, PrivateKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateEndpointError {
    #[error(transparent)]
    RcGen(#[from] RcgenError),
    #[error(transparent)]
    Rustls(#[from] rustls::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn generate_certificate(
    alt_names: impl Into<Vec<String>>,
) -> Result<(Certificate, PrivateKey), RcgenError> {
    let certificate = rcgen::generate_simple_self_signed(alt_names)?;
    Ok((
        Certificate(certificate.serialize_der()?),
        PrivateKey(certificate.serialize_private_key_der()),
    ))
}

pub fn create_endpoint() -> Result<Endpoint, CreateEndpointError> {
    let (certificate, private_key) = generate_certificate(vec!["::1".into()])?;

    let server_config = quinn::ServerConfig::with_single_cert(vec![certificate], private_key)?;
    // let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    // transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = crate::server(server_config, crate::IPV6_WILDCARD_DEFAULT_PORT)?;

    Ok(endpoint)
}
