# Security Policy

## Supported Versions

**During Development (Current Phase):**
We only support the latest release version of DungeonRS with security updates. Once we reach stable release (1.0+), we'll maintain security support for the latest major version.

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you discover a security vulnerability in DungeonRS, please report it privately through GitHub Security Advisories:

1. Go to the [Security tab](https://github.com/dungeon-rs/dungeon-rs/security) of this repository
2. Click "Report a vulnerability"
3. Fill out the advisory form with details

## What to Include

When reporting a vulnerability, please include:

- **Description** of the vulnerability
- **Steps to reproduce** the issue
- **Potential impact** and attack scenarios
- **Suggested fixes** (if you have any)
- **Your contact information** for follow-up questions

## Response Process

1. **Acknowledgement**: We'll acknowledge receipt of your report within 48 hours
2. **Investigation**: We'll investigate and assess the vulnerability
3. **Timeline**: We'll provide an estimated timeline for a fix
4. **Updates**: We'll keep you informed of our progress
5. **Resolution**: We'll notify you when the vulnerability is resolved
6. **Disclosure**: We'll coordinate responsible disclosure timing with you

## Security Considerations

DungeonRS processes various file types and user-generated content. Areas of particular security importance include:

### File Processing
- **Asset loading**: Images, scripts, and other media files
- **Project files**: JSON/TOML serialisation and deserialization
- **Export functionality**: Generated map files and images

### Script Execution
- **Rhai scripts**: For asset filtering and processing
- **Custom shaders**: WebGL/graphics pipeline security

## Scope

This security policy covers:
- The DungeonRS application and its core libraries
- Asset processing and project file handling
- Build and distribution processes

## Out of Scope

The following are generally not considered security vulnerabilities:
- Issues requiring physical access to the user's machine
- Social engineering attacks
- Vulnerabilities in third-party dependencies (please report these upstream)
- Performance issues without security implications

## Recognition

We appreciate responsible disclosure and will publicly acknowledge security researchers who help improve DungeonRS security (with your permission).

## Contact

For non-security related issues, please use our standard [issue reporting process](https://github.com/dungeon-rs/dungeon-rs/issues).

For general questions about this security policy, you can reach out via [GitHub Discussions](https://github.com/dungeon-rs/dungeon-rs/discussions).
