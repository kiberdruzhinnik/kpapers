# Tasks

## 1. Public download destination validation

- [x] 1.1 Replace provider-specific host checks for PDF download URLs with HTTPS and public-address validation; cover IPv4, IPv6, mapped IPv4, and special-use ranges with focused tests.
- [x] 1.2 Ensure connections use validated resolved addresses while preserving TLS hostname validation; verify with a controlled resolver that a hostname cannot change to a private address between validation and connection.
- [x] 1.3 Apply the same address and HTTPS checks to every redirect while retaining redirect limits; verify redirects to public hosts succeed and private or non-HTTPS destinations are rejected.
- [x] 1.4 Verify supported provider APIs and public folder URL validation remain unchanged, and exercise a public provider-issued download host outside the existing allowlists.

## 2. Per-file actionable failure reporting

- [x] 2.1 Preserve safe, specific failure stage and reason for each failed document; verify stored errors omit URL credentials and signed query secrets.
- [x] 2.2 Include each failed filename and its specific stored error in scheduled refresh summaries; verify multiple PDF failures remain distinguishable.
- [x] 2.3 Verify PDF review displays the per-file failure detail and successful retry clears it.
