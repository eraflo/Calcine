//! Authenticode check of a downloaded installer, through PowerShell's
//! `Get-AuthenticodeSignature` (no native code needed).

use std::path::Path;

use calcine_core::{Error, Result};

/// The publisher an installer must be signed by, when it is signed.
const EXPECTED_PUBLISHER: &str = "Qualcomm";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signature {
    /// Valid signature from Qualcomm.
    Signed { subject: String },
    /// No signature. Qualcomm doesn't sign GenieX installers yet; the
    /// SHA-256 from the official manifest (fetched over HTTPS) is checked
    /// instead.
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
        "Valid" if subject.contains(EXPECTED_PUBLISHER) => Ok(Signature::Signed { subject }),
        "Valid" => Err(Error::InvalidInput(format!(
            "the GenieX installer is signed by someone else ({subject}); it wasn't installed"
        ))),
        other => Err(Error::InvalidInput(format!(
            "the GenieX installer's signature is invalid ({other}); it wasn't installed"
        ))),
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
        let output = "Valid\r\nCN=Qualcomm Technologies, Inc., O=Qualcomm Technologies, Inc.\r\n";
        assert!(matches!(parse(output).unwrap(), Signature::Signed { .. }));
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
