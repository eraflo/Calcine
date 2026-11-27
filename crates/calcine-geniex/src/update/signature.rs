//! Authenticode check of a downloaded installer, through PowerShell's
//! `Get-AuthenticodeSignature` (no native code needed).

use std::path::Path;

use calcine_core::{Error, Result};

/// The organizations (the certificate's `O=`) Qualcomm signs as, compared
/// without case: `QUALCOMM Inc.` signs GenieX installers since 0.12, and
/// Microsoft Trusted Signing issues `Qualcomm` certificates.
const QUALCOMM_ORGANIZATIONS: &[&str] =
    &["QUALCOMM Inc.", "Qualcomm Technologies, Inc.", "Qualcomm"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signature {
    /// Valid signature from Qualcomm.
    Signed { subject: String },
    /// No signature: GenieX installers before 0.12. The SHA-256 from the
    /// official manifest (fetched over HTTPS) is checked instead.
    Unsigned,
    /// Not checked on this OS.
    NotChecked,
}

/// Refuse tampered or foreign-signed installers.
pub async fn verify(path: &Path) -> Result<Signature> {
    if !cfg!(windows) {
        return Ok(Signature::NotChecked);
    }
    let mut command = tokio::process::Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            // The path comes through the environment, never the script text.
            "$s = Get-AuthenticodeSignature -LiteralPath $env:CALCINE_INSTALLER; \
             $s.Status.ToString(); if ($s.SignerCertificate) { $s.SignerCertificate.Subject }",
        ])
        .env("CALCINE_INSTALLER", path)
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    let output = command.output().await?;
    if !output.status.success() {
        return Err(Error::Command {
            command: "Get-AuthenticodeSignature".into(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    parse(&String::from_utf8_lossy(&output.stdout))
}

fn parse(output: &str) -> Result<Signature> {
    let mut lines = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let status = lines.next().unwrap_or_default();
    let subject = lines.next().unwrap_or_default().to_owned();
    match status {
        "NotSigned" => Ok(Signature::Unsigned),
        "Valid" if is_qualcomm(&subject) => Ok(Signature::Signed { subject }),
        "Valid" => Err(Error::InvalidInput(format!(
            "the GenieX installer is signed by someone else ({subject}); it wasn't installed"
        ))),
        other => Err(Error::InvalidInput(format!(
            "the GenieX installer's signature is invalid ({other}); it wasn't installed"
        ))),
    }
}

fn is_qualcomm(subject: &str) -> bool {
    organization(subject).is_some_and(|organization| {
        QUALCOMM_ORGANIZATIONS
            .iter()
            .any(|known| organization.eq_ignore_ascii_case(known))
    })
}

/// The `O=` attribute of a certificate subject as .NET prints it
/// (`CN=…, O="Name, Inc.", L=…`: values with a comma are quoted).
fn organization(subject: &str) -> Option<&str> {
    let mut rest = subject;
    loop {
        let (key, after) = rest.split_once('=')?;
        let (value, next) = match after.strip_prefix('"') {
            Some(quoted) => {
                let (value, next) = quoted.split_once('"')?;
                (value, next.trim_start_matches(',').trim_start())
            }
            None => after.split_once(", ").unwrap_or((after, "")),
        };
        if key.trim().eq_ignore_ascii_case("O") {
            return Some(value.trim());
        }
        if next.is_empty() {
            return None;
        }
        rest = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_installers_pass_on_their_checksum() {
        assert_eq!(parse("NotSigned\n").unwrap(), Signature::Unsigned);
    }

    #[test]
    fn qualcomm_signatures_pass() {
        for subject in [
            // GenieX 0.12's installer.
            "CN=QUALCOMM Inc., OU=Windows Mobile Integration, O=QUALCOMM Inc., L=San Diego, \
             S=California, C=US, SERIALNUMBER=2271160, OID.2.5.4.15=Private Organization, \
             OID.1.3.6.1.4.1.311.60.2.1.2=Delaware, OID.1.3.6.1.4.1.311.60.2.1.3=US",
            // Microsoft Trusted Signing (Qualcomm's own tools).
            "CN=Qualcomm, O=Qualcomm, L=San Diego, S=California, C=US",
            r#"CN="Qualcomm Technologies, Inc.", O="Qualcomm Technologies, Inc.", C=US"#,
        ] {
            let output = format!("Valid\r\n{subject}\r\n");
            assert!(
                matches!(parse(&output).unwrap(), Signature::Signed { .. }),
                "{subject}"
            );
        }
    }

    #[test]
    fn only_the_organization_counts() {
        assert!(!is_qualcomm("CN=Qualcomm, O=Someone Else, C=US"));
        assert!(!is_qualcomm("CN=Someone, O=Qualcomm Lookalike Ltd, C=US"));
        assert!(!is_qualcomm(r#"CN=Someone, OU="Qualcomm, Inc.", C=US"#));
        assert!(!is_qualcomm("CN=Qualcomm"));
    }

    #[test]
    fn tampered_or_foreign_installers_fail() {
        assert!(parse("HashMismatch\n").is_err());
        assert!(parse("Valid\nCN=Someone Else\n").is_err());
        assert!(parse("").is_err());
    }

    #[tokio::test]
    async fn reads_a_real_file() {
        // Any unsigned file: this source file is plain text.
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let signature = verify(&path).await;
        // Text files are "UnknownError" or "NotSigned" depending on Windows.
        if cfg!(windows) {
            assert!(
                matches!(
                    signature,
                    Ok(Signature::Unsigned) | Err(Error::InvalidInput(_))
                ),
                "{signature:?}"
            );
        }
    }
}
