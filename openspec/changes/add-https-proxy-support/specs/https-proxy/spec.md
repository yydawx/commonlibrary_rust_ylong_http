## ADDED Requirements

### Requirement: HTTPS Proxy Connection via TLS

The system SHALL support connecting through HTTPS proxies where the proxy server itself uses TLS encryption. The client SHALL establish TLS with the proxy before sending the HTTP CONNECT request.

#### Scenario: Connect to HTTP target via HTTPS proxy
- **WHEN** a request to `http://example.com` is made through an HTTPS proxy at `https://proxy.example.com:443`
- **THEN** the system SHALL first establish TLS with the proxy server
- **AND** the system SHALL send HTTP CONNECT through the encrypted proxy connection
- **AND** the system SHALL receive HTTP 200 from the proxy
- **AND** the system SHALL establish a TCP tunnel through the proxy
- **AND** the system SHALL send the HTTP request to the target server through the tunnel

#### Scenario: Connect to HTTPS target via HTTPS proxy
- **WHEN** a request to `https://example.com` is made through an HTTPS proxy at `https://proxy.example.com:443`
- **THEN** the system SHALL first establish TLS with the proxy server
- **AND** the system SHALL send HTTP CONNECT through the encrypted proxy connection
- **AND** the system SHALL establish target TLS with `example.com`
- **AND** the system SHALL send the HTTPS request through the proxy tunnel

### Requirement: Proxy CA Certificate Configuration

The system SHALL allow configuration of a custom CA certificate for verifying the proxy server's certificate. When a CA certificate is provided, the system SHALL use it to verify the proxy certificate during TLS handshake.

#### Scenario: Proxy with custom CA certificate
- **WHEN** a proxy is configured with `proxy_ca_file("corporate-ca.pem")`
- **AND** a connection is made through that proxy
- **THEN** the system SHALL use the configured CA certificate to verify the proxy's certificate
- **AND** the TLS handshake SHALL fail if the proxy certificate is not signed by the configured CA

#### Scenario: Proxy with default system CA
- **WHEN** a proxy is configured without a CA certificate
- **AND** a connection is made through that proxy
- **THEN** the system SHALL use the default system CA certificates to verify the proxy's certificate

### Requirement: Proxy Certificate Verification Control

The system SHALL provide an option to skip proxy certificate verification for development or testing purposes. When enabled, the system SHALL accept any proxy certificate regardless of validity.

#### Scenario: Skip proxy certificate verification
- **WHEN** a proxy is configured with `danger_accept_invalid_proxy_certs(true)`
- **AND** a connection is made through that proxy
- **THEN** the system SHALL accept any certificate presented by the proxy server
- **AND** the system SHALL complete the TLS handshake without certificate validation errors

#### Scenario: Verify proxy certificate (default)
- **WHEN** a proxy is configured with `danger_accept_invalid_proxy_certs(false)` (default)
- **AND** the proxy presents an invalid certificate
- **THEN** the system SHALL fail the TLS handshake with a certificate verification error

### Requirement: HTTPS Proxy Detection

The system SHALL detect HTTPS proxies based on the proxy URL scheme. When the proxy URL scheme is `https`, the system SHALL treat the proxy as an HTTPS proxy and perform proxy TLS establishment before CONNECT.

#### Scenario: HTTPS proxy detected by scheme
- **WHEN** a proxy is created with `Proxy::all("https://proxy.example.com:443")`
- **THEN** the system SHALL identify this as an HTTPS proxy
- **AND** the system SHALL attempt to establish TLS with the proxy before sending CONNECT

#### Scenario: HTTP proxy detected by scheme
- **WHEN** a proxy is created with `Proxy::all("http://proxy.example.com:8080")`
- **THEN** the system SHALL identify this as an HTTP proxy
- **AND** the system SHALL NOT establish TLS with the proxy
- **AND** the system SHALL send CONNECT over the raw TCP connection

### Requirement: SNI for Proxy TLS

The system SHALL set the Server Name Indication (SNI) to the proxy hostname during proxy TLS handshake.

#### Scenario: SNI set from proxy URL
- **WHEN** a connection is made through a proxy at `https://proxy.example.com:443`
- **THEN** the system SHALL set SNI to `proxy.example.com` in the TLS ClientHello
- **AND** the proxy SHALL receive a certificate valid for `proxy.example.com`
