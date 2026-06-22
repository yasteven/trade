// build.rs
use rcgen::{CertificateParams, KeyPair, DnType, SanType};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
// usta/build.rs


/// Generates self-signed certs/keys **without SANs** (for simple LAN testing).
/// Use this if:
/// - You only need a single hostname (e.g., `localhost`).
/// - You don’t care about strict hostname verification.
/// - You’re prototyping or testing locally.
fn generate_certs_and_copy_to_binary_dir() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let cert_path = Path::new(&out_dir).join("cert.pem");
    let key_path = Path::new(&out_dir).join("key.pem");

    if !cert_path.exists() || !key_path.exists() {
        let mut params = CertificateParams::new(vec!["localhost".to_string()]);
        params.distinguished_name.push(DnType::OrganizationName("LAN QUIC Mesh".to_string()));
        params.key_pair = KeyPair::generate().expect("Failed to generate key pair");
        params.not_before = chrono::Utc::now();
        params.not_after = chrono::Utc::now() + chrono::Duration::days(365 * 10);

        let cert = rcgen::Certificate::from_params(params).expect("Failed to generate certificate");

        File::create(&cert_path)
            .and_then(|mut f| f.write_all(cert.pem().as_bytes()))
            .expect("Failed to write cert.pem");
        File::create(&key_path)
            .and_then(|mut f| f.write_all(cert.serialize_private_key_pem().as_bytes()))
            .expect("Failed to write key.pem");

        println!("Generated cert.pem and key.pem (no SANs) in {}", out_dir);
    }

    // Copy to binary directory (target/debug or target/release)
    let target_dir = Path::new(
        &std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string())
    ).join(std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()));
    fs::create_dir_all(&target_dir).expect("Failed to create target directory");
    fs::copy(&cert_path, target_dir.join("cert.pem")).expect("Failed to copy cert.pem");
    fs::copy(&key_path, target_dir.join("key.pem")).expect("Failed to copy key.pem");
}

/// Generates self-signed certs/keys **with SANs** (for production or multi-host LAN).
/// Use this if:
/// - You need to support multiple hostnames/IPs (e.g., `node1.local`, `192.168.1.100`).
/// - You want strict hostname verification in your QUIC mesh.
/// - You’re deploying to a real LAN with multiple services.
fn _generate_certs_with_sans_and_copy_to_binary_dir() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let cert_path = Path::new(&out_dir).join("cert_with_sans.pem");
    let key_path = Path::new(&out_dir).join("key_with_sans.pem");

    if !cert_path.exists() || !key_path.exists() {
        // Define SANs (add all hostnames/IPs for your LAN)
        let san_list = vec![
            SanType::DnsName("lan.quic.mesh".to_string()),
            SanType::DnsName("node1.local".to_string()),
            SanType::DnsName("node2.local".to_string()),
            SanType::IpAddress("192.168.1.100".parse().unwrap()),
            SanType::IpAddress("192.168.1.101".parse().unwrap()),
        ];

        let mut params = CertificateParams::new(vec!["lan.quic.mesh".to_string()]);
        params.san = san_list;
        params.distinguished_name.push(DnType::OrganizationName("LAN QUIC Mesh".to_string()));
        params.key_pair = KeyPair::generate().expect("Failed to generate key pair");
        params.not_before = chrono::Utc::now();
        params.not_after = chrono::Utc::now() + chrono::Duration::days(365 * 10);

        let cert = rcgen::Certificate::from_params(params).expect("Failed to generate certificate");

        File::create(&cert_path)
            .and_then(|mut f| f.write_all(cert.pem().as_bytes()))
            .expect("Failed to write cert_with_sans.pem");
        File::create(&key_path)
            .and_then(|mut f| f.write_all(cert.serialize_private_key_pem().as_bytes()))
            .expect("Failed to write key_with_sans.pem");

        println!("Generated cert_with_sans.pem and key_with_sans.pem in {}", out_dir);
    }

    // Copy to binary directory (target/debug or target/release)
    let target_dir = Path::new(
        &std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string())
    ).join(std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()));
    fs::create_dir_all(&target_dir).expect("Failed to create target directory");
    fs::copy(&cert_path, target_dir.join("cert_with_sans.pem")).expect("Failed to copy cert_with_sans.pem");
    fs::copy(&key_path, target_dir.join("key_with_sans.pem")).expect("Failed to copy key_with_sans.pem");
}


fn main() {
    // Get the target directory (e.g., /home/seev0/etc/trade/bin/alpine_usta/release)
    let target_dir = Path::new(
        &std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string())
    ).join(std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()));

    // Define paths for cert.pem and key.pem in the target directory
    let cert_path = target_dir.join("cert.pem");
    let key_path = target_dir.join("key.pem");

    // Generate the certificate and key if they don't exist
    if !cert_path.exists() || !key_path.exists() {
        let mut params = CertificateParams::new(vec!["localhost".to_string()]);
        params.distinguished_name.push(DnType::OrganizationName("LAN QUIC Mesh".to_string()));
        params.key_pair = KeyPair::generate().expect("Failed to generate key pair");
        params.not_before = chrono::Utc::now();
        params.not_after = chrono::Utc::now() + chrono::Duration::days(365 * 10);

        let cert = rcgen::Certificate::from_params(params).expect("Failed to generate certificate");

        File::create(&cert_path)
            .and_then(|mut f| f.write_all(cert.pem().as_bytes()))
            .expect("Failed to write cert.pem");
        File::create(&key_path)
            .and_then(|mut f| f.write_all(cert.serialize_private_key_pem().as_bytes()))
            .expect("Failed to write key.pem");

        println!("Generated cert.pem and key.pem in {}", target_dir.display());
    }

    // Tell Cargo to re-run this build script if the files are missing
    println!("cargo:rerun-if-changed=build.rs");
}
