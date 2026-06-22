// usta/build.rs

use rcgen::{generate_simple_self_signed, CertificateParams, CertifiedKey, DnType, KeyPair};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Change this to true when you want the LAN/SAN cert instead of localhost-only.
const USE_SAN_CERT: bool = false;

fn main() {
    if USE_SAN_CERT {
        generate_certs_with_sans_and_copy_to_binary_dir();
    } else {
        generate_simple_localhost_certs_and_copy_to_binary_dir();
    }

    println!("cargo:rerun-if-changed=build.rs");
}

fn target_profile_dir() -> PathBuf {
    Path::new(&std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string()))
        .join(std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()))
}

fn write_and_copy_cert_pair(
    cert_filename: &str,
    key_filename: &str,
    cert_pem: &str,
    key_pem: &str,
) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR missing");

    let cert_path = Path::new(&out_dir).join(cert_filename);
    let key_path = Path::new(&out_dir).join(key_filename);

    File::create(&cert_path)
        .and_then(|mut f| f.write_all(cert_pem.as_bytes()))
        .expect("Failed to write cert PEM");

    File::create(&key_path)
        .and_then(|mut f| f.write_all(key_pem.as_bytes()))
        .expect("Failed to write key PEM");

    let target_dir = target_profile_dir();
    fs::create_dir_all(&target_dir).expect("Failed to create target directory");

    fs::copy(&cert_path, target_dir.join(cert_filename)).expect("Failed to copy cert PEM");
    fs::copy(&key_path, target_dir.join(key_filename)).expect("Failed to copy key PEM");

    println!(
        "Generated {} and {} in {}",
        cert_filename,
        key_filename,
        target_dir.display()
    );
}

/// Example 1:
/// Generates a simple self-signed localhost certificate.
///
/// Output:
/// - cert.pem
/// - key.pem
fn generate_simple_localhost_certs_and_copy_to_binary_dir() {
    let target_dir = target_profile_dir();
    let cert_path = target_dir.join("cert.pem");
    let key_path = target_dir.join("key.pem");

    if cert_path.exists() && key_path.exists() {
        return;
    }

    let subject_alt_names = vec!["localhost".to_string()];

    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(subject_alt_names)
            .expect("Failed to generate localhost self-signed certificate");

    write_and_copy_cert_pair(
        "cert.pem",
        "key.pem",
        &cert.pem(),
        &signing_key.serialize_pem(),
    );
}

/// Example 2:
/// Generates a self-signed LAN/SAN certificate.
///
/// Add/remove hostnames and IP strings here as needed.
///
/// Output:
/// - cert.pem
/// - key.pem
fn generate_certs_with_sans_and_copy_to_binary_dir() {
    let target_dir = target_profile_dir();
    let cert_path = target_dir.join("cert.pem");
    let key_path = target_dir.join("key.pem");

    if cert_path.exists() && key_path.exists() {
        return;
    }

    let subject_alt_names = vec![
        "localhost".to_string(),
        "lan.quic.mesh".to_string(),
        "node1.local".to_string(),
        "node2.local".to_string(),
        "192.168.1.100".to_string(),
        "192.168.1.101".to_string(),
        "127.0.0.1".to_string(),
    ];

    let mut params =
        CertificateParams::new(subject_alt_names).expect("Failed to create certificate params");

    params
        .distinguished_name
        .push(DnType::OrganizationName, "LAN QUIC Mesh");

    let signing_key = KeyPair::generate().expect("Failed to generate key pair");

    let cert = params
        .self_signed(&signing_key)
        .expect("Failed to generate SAN self-signed certificate");

    write_and_copy_cert_pair(
        "cert.pem",
        "key.pem",
        &cert.pem(),
        &signing_key.serialize_pem(),
    );
}
