# Container Image Security

This document outlines the security measures implemented for container images used in the InheritX backend, specifically addressing image scanning and signing as part of our CI/CD pipeline.

## Vulnerability Scanning

All Docker images are scanned for vulnerabilities before deployment using [Trivy](https://aquasecurity.github.io/trivy/).

### CI/CD Integration
- **Workflow:** The scanning is integrated into GitHub Actions (`.github/workflows/image-scanning.yml`).
- **Thresholds:** The pipeline is configured to **block deployment** (exit code 1) if any vulnerabilities with a severity of `HIGH` or `CRITICAL` are detected.
- **Reporting:** Scan results are uploaded in SARIF format to the GitHub Security tab, providing centralized vulnerability alerts and tracking.

## Image Signing

To ensure the integrity and provenance of our container images, we use [Cosign](https://github.com/sigstore/cosign) for image signing.

### Process
1. **Build:** The Docker image is built during the CI run.
2. **Scan:** The image is scanned for vulnerabilities.
3. **Sign:** If the scan passes, the image is pushed to the container registry and signed using keyless signing via OIDC in GitHub Actions.

This process ensures that only verified, vulnerability-free images are deployed to the production environment.
