# Security Policy

## Supported Versions

Currently, only the latest version of `main` is supported with security updates.

## Reporting a Vulnerability

Please report any security vulnerabilities by creating an issue or contacting the maintainers directly.

## Security Scanning

This repository uses automated security scanning integrated into the CI/CD pipeline to detect vulnerabilities before deployment.

- **Rust/Backend**: Uses `cargo-clippy` to enforce strict security lints and `cargo audit` to detect vulnerabilities in dependencies.
- **Node.js/Frontend**: Uses `npm audit` to detect vulnerable dependencies and enforces a threshold of `high` and `critical`.
- **SAST**: GitHub CodeQL is run on all PRs and pushes to the main branch to perform Static Application Security Testing on the codebase.
- **Container Images**: The `aquasecurity/trivy-action` is used to scan container images for vulnerabilities before deployment. Results are uploaded as SARIF to the GitHub Security tab.
