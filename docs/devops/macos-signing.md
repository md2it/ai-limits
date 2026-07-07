# macOS GitHub Signing

## GitHub Actions Behavior

The desktop workflow builds macOS as a signed universal Apple app:

```text
npm exec tauri -- build --bundles app --target universal-apple-darwin
```

Workflow:

```text
.github/workflows/desktop-build.yml
```

Default mode:

```text
sign-only
```

Modes:

- `sign-only`: Developer ID signed, not notarized;
- `submit-only`: signed and submitted to Apple notarization without waiting for
  stapling;
- `full`: signed, notarized, and stapled.

## Secrets

Required for signing:

```text
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
KEYCHAIN_PASSWORD
```

Required for `submit-only` and `full` notarization:

```text
APPLE_API_KEY_CONTENT
APPLE_API_KEY_ID
APPLE_API_ISSUER
```

Example file:

```text
scripts/macos-signing-secrets.example
```

Do not set `APPLE_SIGNING_IDENTITY` in GitHub secrets when using
`APPLE_CERTIFICATE`. The workflow imports the `.p12` certificate into a
temporary keychain, and Tauri derives the signing identity from that
certificate.
