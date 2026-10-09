//! The certificate the network port presents: self-signed, made on this PC
//! the first time, kept in Calcine's data folder. Clients trust it by its
//! SHA-256 fingerprint (pinning) or by importing `certificate.pem`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha2::{Digest, Sha256};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::rustls::crypto::ring;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

/// A certificate and its private key.
pub struct Identity {
    certificate: Vec<u8>,
    key: Vec<u8>,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("fingerprint", &self.fingerprint())
            .finish_non_exhaustive()
    }
}

const CERTIFICATE: &str = "certificate.der";
const KEY: &str = "key.der";
/// The certificate again, in the format clients import.
const CERTIFICATE_PEM: &str = "certificate.pem";

impl Identity {
    /// The certificate in `dir`, made first if there's none.
    pub fn load_or_create(dir: &Path, names: &[String]) -> Result<Self, String> {
        let certificate = std::fs::read(dir.join(CERTIFICATE));
        let key = std::fs::read(dir.join(KEY));
        if let (Ok(certificate), Ok(key)) = (certificate, key) {
            return Ok(Self { certificate, key });
        }
        Self::create(dir, names)
    }

    /// A new certificate for `names` (host names and addresses), replacing
    /// the one in `dir`.
    pub fn create(dir: &Path, names: &[String]) -> Result<Self, String> {
        let failed = |err: &dyn std::fmt::Display| format!("couldn't make a certificate: {err}");
        let mut names = names.to_vec();
        names.push("localhost".into());
        names.dedup();
        let key = rcgen::KeyPair::generate().map_err(|err| failed(&err))?;
        let mut params = rcgen::CertificateParams::new(names).map_err(|err| failed(&err))?;
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, "Calcine local network API");
        // Valid from the start of this year for five years.
        let year = current_year();
        params.not_before = rcgen::date_time_ymd(year, 1, 1);
        params.not_after = rcgen::date_time_ymd(year + 5, 1, 1);
        let certificate = params.self_signed(&key).map_err(|err| failed(&err))?;

        std::fs::create_dir_all(dir).map_err(|err| failed(&err))?;
        let write = |name: &str, bytes: &[u8]| {
            std::fs::write(dir.join(name), bytes).map_err(|err| failed(&err))
        };
        write(CERTIFICATE, certificate.der())?;
        write(KEY, &key.serialize_der())?;
        write(CERTIFICATE_PEM, certificate.pem().as_bytes())?;
        Ok(Self {
            certificate: certificate.der().to_vec(),
            key: key.serialize_der(),
        })
    }

    /// SHA-256 of the certificate, `AB:CD:…`, as browsers and `openssl` show it.
    pub fn fingerprint(&self) -> String {
        Sha256::digest(&self.certificate)
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<Vec<_>>()
            .join(":")
    }

    /// Where clients find the certificate to import.
    pub fn pem_path(dir: &Path) -> PathBuf {
        dir.join(CERTIFICATE_PEM)
    }

    /// TLS 1.2 and 1.3, HTTP/2 or HTTP/1.1.
    pub fn acceptor(&self) -> Result<TlsAcceptor, String> {
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.key.clone()));
        let mut config = ServerConfig::builder_with_provider(Arc::new(ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|err| err.to_string())?
            .with_no_client_auth()
            .with_single_cert(vec![CertificateDer::from(self.certificate.clone())], key)
            .map_err(|err| format!("the certificate can't be used: {err}"))?;
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        Ok(TlsAcceptor::from(Arc::new(config)))
    }
}

fn current_year() -> i32 {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() / 86_400);
    // Close enough: the certificate starts on January 1st anyway.
    1970 + i32::try_from(days * 400 / 146_097).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_a_certificate_once_and_keeps_it() {
        let dir = std::env::temp_dir().join(format!("calcine-tls-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let first =
            Identity::load_or_create(&dir, &["calcine-pc".into(), "192.168.1.5".into()]).unwrap();
        assert_eq!(first.fingerprint().len(), 32 * 3 - 1);
        assert!(Identity::pem_path(&dir).is_file());
        assert!(first.acceptor().is_ok());
        let again = Identity::load_or_create(&dir, &[]).unwrap();
        assert_eq!(again.fingerprint(), first.fingerprint());
        let new = Identity::create(&dir, &[]).unwrap();
        assert_ne!(new.fingerprint(), first.fingerprint());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
