# Security policy

Calcine exposes a local HTTP API (`127.0.0.1:18181`) that runs models on your
machine, and optionally the same API to your other devices over HTTPS (port
18443, off by default). We take issues that could let a web page, another
program, or a remote host reach that API without a valid key, read local files,
or tamper with updates seriously.

## Supported versions

| Version | Supported |
|---|---|
| Latest stable release (`main`) | ✅ |
| Latest beta (`dev`) | ✅ best effort |
| Older releases | ❌ |

## Reporting a vulnerability

Please **do not open a public issue**. Report it privately through
[GitHub private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
("Security" tab → "Report a vulnerability").

Include the Calcine and GenieX versions, your OS, steps to reproduce, and the
impact you observed. We aim to acknowledge reports within 72 hours and to ship
a fix for confirmed high-severity issues within 14 days.

Vulnerabilities in GenieX itself should be reported to
[qualcomm/GenieX](https://github.com/qualcomm/GenieX).
