# Security Policy

## Supported Versions

DuckPet is pre-1.0 software. Security fixes are handled on the latest `main` branch unless a release branch is explicitly maintained.

## Reporting a Vulnerability

Please do not open a public issue for a security vulnerability.

Use a private GitHub security advisory if the repository has advisories enabled. If not, contact the maintainer privately and include:

- A clear description of the issue.
- Steps to reproduce.
- Impact and affected versions, if known.
- Any proof-of-concept details needed to verify the report.

## Scope

Useful security reports include:

- Unsafe file or path handling.
- Unexpected command execution.
- Dangerous Tauri permissions.
- WebView injection issues.
- Insecure update or download behavior.
- Credential or secret exposure.

Reports about unsigned installers or Windows SmartScreen warnings are expected for current builds unless code signing is configured.
