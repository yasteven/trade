
// trade/vsta/src/lens/v_dr_r/cert.rs

use std::sync::Arc;
use std::fs::File;
use std::io::{BufReader};

use rustls::client::danger::ServerCertVerifier;
use rustls::pki_types::CertificateDer;
use rustls::client::danger::ServerCertVerified;
use rustls::pki_types::ServerName;
use rustls::pki_types::UnixTime;

use rustls::client::danger::HandshakeSignatureValid;
use rustls::DigitallySignedStruct;
use rustls::SignatureScheme;

use rustls_pemfile;

#[derive(Debug)]
pub struct SelfSignedVerifier {
  trusted_cert: CertificateDer<'static>
}

impl SelfSignedVerifier {
  pub fn new(trusted_cert: CertificateDer<'static>) -> Arc<Self> {
    Arc::new(Self { trusted_cert })
  }
}

impl ServerCertVerifier for SelfSignedVerifier {
  fn verify_server_cert(
    &self,
    end_entity: &CertificateDer,
    _intermediates: &[CertificateDer],
    _server_name: &ServerName,
    _ocsp_response: &[u8],
    _now: UnixTime
  ) -> Result<ServerCertVerified, rustls::Error> {
    if *end_entity == self.trusted_cert {
      Ok(ServerCertVerified::assertion())
    } else {
      Err(
        rustls::Error::InvalidCertificate(
          rustls::CertificateError::Other(
            rustls::OtherError(
              Arc::new(
                std::io::Error::new(
                  std::io::ErrorKind::InvalidData,
                  "Server certificate does not match trusted certificate"
                )
              )
            )
          )
        )
      )
    }
  }

  fn verify_tls12_signature(
    &self,
    _message: &[u8],
    _cert: &CertificateDer,
    _dss: &DigitallySignedStruct
  ) -> Result<HandshakeSignatureValid, rustls::Error> {
    Ok(HandshakeSignatureValid::assertion())
  }

  fn verify_tls13_signature(
    &self,
    _message: &[u8],
    _cert: &CertificateDer,
    _dss: &DigitallySignedStruct
  ) -> Result<HandshakeSignatureValid, rustls::Error> {
    Ok(HandshakeSignatureValid::assertion())
  }

  fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
    vec![
      SignatureScheme::RSA_PSS_SHA512
    ]
  }
}

pub fn example_client_load_trusted_cert() -> Result<CertificateDer<'static>, Box<dyn std::error::Error>> {
  let cert_file = File::open("cert.pem")?;
  let mut cert_reader = BufReader::new(cert_file);
  let mut certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
    .collect::<Result<Vec<_>, _>>()?;
  if certs.is_empty() {
    return Err("No certificates found in cert.pem".into());
  }
  Ok(certs.remove(0))
}