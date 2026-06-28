// trade/wsta/src/seek_bridge.rs
//
// Existing dsta seek! mesh bridge.
// This replaces the Iced frontend's direct connection tasks while preserving
// the same generated channel names.

use std::sync::Arc;
use std::time::Duration;

use quinn::crypto::rustls::QuicClientConfig;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum SeekBridgeCommand {
    SendSnively(dsta::Snively),
}

#[derive(Debug)]
pub enum SeekBridgeEvent {
    Connected { remote: String },
    Disconnected { reason: String },
    DebugLine(String),
    MadeBot(dsta::MadeBot),
    GotTtai(dsta::GotTtai),
    GotTick00(dsta::GotTick00),
    GotTick01(dsta::GotTick01),
}

#[derive(Debug)]
struct SkipAllVerification;

impl ServerCertVerifier for SkipAllVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA256,
        ]
    }
}

pub fn spawn_seek_bridge(
    backend_addr: String,
) -> (
    mpsc::Sender<SeekBridgeCommand>,
    mpsc::Receiver<SeekBridgeEvent>,
) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<SeekBridgeCommand>(8192);
    let (event_tx, event_rx) = mpsc::channel::<SeekBridgeEvent>(8192);

    tokio::spawn(run_seek_bridge(backend_addr, cmd_rx, event_tx));

    (cmd_tx, event_rx)
}

async fn run_seek_bridge(
    backend_addr: String,
    mut cmd_rx: mpsc::Receiver<SeekBridgeCommand>,
    event_tx: mpsc::Sender<SeekBridgeEvent>,
) {
    let _ = event_tx
        .send(SeekBridgeEvent::DebugLine(format!(
            "seek bridge connecting to {}",
            backend_addr
        )))
        .await;

    let mut handle = match connect_seek(&backend_addr).await {
        Ok(handle) => handle,
        Err(e) => {
            let _ = event_tx
                .send(SeekBridgeEvent::Disconnected { reason: e })
                .await;
            return;
        }
    };

    let _ = event_tx
        .send(SeekBridgeEvent::Connected {
            remote: backend_addr.clone(),
        })
        .await;

    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    SeekBridgeCommand::SendSnively(snively) => {
                        if let Err(e) = handle.to_dude_snively.send(snively).await {
                            let _ = event_tx.send(SeekBridgeEvent::Disconnected {
                                reason: format!("to_dude_snively send failed: {}", e),
                            }).await;
                            break;
                        }
                    }
                }
            }

            got = handle.from_dude_madebot.recv() => {
                match got {
                    Some(value) => {
                        let _ = event_tx.send(SeekBridgeEvent::MadeBot(value)).await;
                    }
                    None => {
                        let _ = event_tx.send(SeekBridgeEvent::Disconnected {
                            reason: "from_dude_madebot closed".to_string(),
                        }).await;
                        break;
                    }
                }
            }

            got = handle.from_dude_gotttai.recv() => {
                match got {
                    Some(value) => {
                        let _ = event_tx.send(SeekBridgeEvent::GotTtai(value)).await;
                    }
                    None => {
                        let _ = event_tx.send(SeekBridgeEvent::Disconnected {
                            reason: "from_dude_gotttai closed".to_string(),
                        }).await;
                        break;
                    }
                }
            }

            got = handle.from_dude_gottick00.recv() => {
                match got {
                    Some(value) => {
                        let _ = event_tx.send(SeekBridgeEvent::GotTick00(value)).await;
                    }
                    None => {
                        let _ = event_tx.send(SeekBridgeEvent::Disconnected {
                            reason: "from_dude_gottick00 closed".to_string(),
                        }).await;
                        break;
                    }
                }
            }

            got = handle.from_dude_gottick01.recv() => {
                match got {
                    Some(value) => {
                        let _ = event_tx.send(SeekBridgeEvent::GotTick01(value)).await;
                    }
                    None => {
                        let _ = event_tx.send(SeekBridgeEvent::Disconnected {
                            reason: "from_dude_gottick01 closed".to_string(),
                        }).await;
                        break;
                    }
                }
            }

            else => {
                let _ = event_tx.send(SeekBridgeEvent::Disconnected {
                    reason: "seek bridge select loop ended".to_string(),
                }).await;
                break;
            }
        }
    }
}

async fn connect_seek(addr: &str) -> Result<dsta::ksta::ConnectionHandle, String> {
    let _ = rustls::crypto::CryptoProvider::install_default(
        rustls::crypto::ring::default_provider(),
    );

    let crypto = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|e| format!("rustls protocol setup failed: {:?}", e))?
    .dangerous()
    .with_custom_certificate_verifier(Arc::new(SkipAllVerification))
    .with_no_client_auth();

    let quic_crypto = QuicClientConfig::try_from(crypto)
        .map_err(|e| format!("rustls -> quinn config failed: {:?}", e))?;

    let mut client_config = quinn::ClientConfig::new(Arc::new(quic_crypto));

    let mut transport_config = quinn::TransportConfig::default();
    transport_config.max_idle_timeout(None);
    client_config.transport_config(Arc::new(transport_config));

    let mut endpoint =
        quinn::Endpoint::client("0.0.0.0:0".parse().map_err(|e| format!("{:?}", e))?)
            .map_err(|e| format!("client endpoint failed: {:?}", e))?;

    endpoint.set_default_client_config(client_config);

    let (connect_tx, connect_rx) = mpsc::channel::<(std::net::SocketAddr, String)>(128);
    let (conn_tx, mut conn_rx) = mpsc::channel(128);

    let mut client_kernel = dsta::ksta::ClientKernel::new(endpoint, connect_rx, conn_tx);

    tokio::spawn(async move {
        if let Err(e) = client_kernel.run().await {
            log::error!("wsta dsta::ksta ClientKernel ended with error: {:?}", e);
        }
    });

    connect_tx
        .send((
            addr.parse()
                .map_err(|e| format!("invalid seek backend addr {}: {:?}", addr, e))?,
            "localhost".to_string(),
        ))
        .await
        .map_err(|e| format!("failed to send seek connect request: {:?}", e))?;

    let handle = tokio::time::timeout(Duration::from_secs(8), conn_rx.recv())
        .await
        .map_err(|_| "timed out waiting for dsta::ksta ConnectionHandle".to_string())?
        .ok_or_else(|| "dsta::ksta ClientKernel exited before ConnectionHandle".to_string())?;

    tokio::time::sleep(Duration::from_millis(300)).await;

    Ok(handle)
}
