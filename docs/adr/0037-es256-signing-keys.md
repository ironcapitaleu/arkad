# ADR 0037: Supabase Auth Signs Login Tokens With ES256 Keys

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-21 ([STA-312 comment `cfd106ee`](https://linear.app/state-machine/issue/STA-312#comment-cfd106ee)) |
| Date | 2026-11-18 |
| Deciders | The project lead |
| Ticket | [STA-312](https://linear.app/state-machine/issue/STA-312) |

## Context

Supabase Auth signs every login token. The `api` crate checks the signature before it trusts the
`sub` claim (STA-315). Three places depend on the key type: `supabase/config.toml` (STA-312), the
token check in `api` (STA-315), and the dev and prod Supabase projects. A later change of the key
type changes the token check and needs a key rotation in each project.

Supabase Auth offers two kinds of key:

- **A shared secret (HS256).** The same secret signs and checks a token. The local stack uses it
  when `config.toml` names no signing key. `supabase start` prints it as `JWT_SECRET`.
- **An asymmetric key pair (ES256 or RS256).** The private key signs. Supabase Auth publishes the
  public key at `/auth/v1/.well-known/jwks.json`, and a public key cannot sign.

## Decision

1. **Supabase Auth signs login tokens with an ES256 key pair.** `auth.signing_keys_path` in
   `supabase/config.toml` names the key file.
2. **The `api` crate reads the public keys from the JWKS endpoint.** It holds no secret for the
   token check.
3. **Each developer generates the local key.** Git ignores `supabase/signing_keys.json`.

## Options Considered

Measured on 2026-11-18 on the local stack, Supabase CLI 3.2.0, on top of `main` at `68a45cf`.

| Option | Who can sign a token | Secret in `api` | Signature size | Supabase CLI |
| ------ | -------------------- | --------------- | -------------- | ------------ |
| **ES256 (chosen)** | Supabase Auth only | None | 64 bytes | "recommended" |
| RS256 | Supabase Auth only | None | 256 bytes with a 2048-bit key | Supported |
| HS256 | Every holder of the secret | The signing secret | 32 bytes | Default without a key file |

- ES256 keeps the signing key in Supabase Auth, as RS256 does, with a smaller signature.
- HS256 puts the signing secret into every service that checks a token. A leak from `api` lets an
  attacker sign a token for any account.
- With HS256, the JWKS endpoint lists no key. The token check then needs the secret in its
  configuration.

## Consequences

- With the ES256 key in place, the local stack signs tokens with `alg` `ES256` and the `kid` of
  the key. The JWKS lists the public key with `key_ops` `verify` and without the private part `d`.
- The `anon` and `service_role` keys that `supabase start` prints are ES256 tokens too.
- Without the key file, `supabase start` stops with a read error. With an empty list `[]` in the
  file, the stack signs with HS256 and the JWKS lists no key, so the token check of `api` rejects
  every token. The README gives the commands that generate the file.
- The decision does not cover the dev and prod projects. A project that runs on the shared secret
  needs a migration to signing keys before launch.

## Open Questions

- Whether a new hosted Supabase project starts on signing keys. MVP Operations answers it, when
  it creates the dev project.

## Sources

- Local run on 2026-11-18: `supabase start`, then
  `curl http://127.0.0.1:54321/auth/v1/.well-known/jwks.json`, and the header and claims of a
  token from `POST /auth/v1/verify`. Two more starts on the same day: one without
  `signing_keys.json`, and one with `[]` in it. The second printed an `anon` key with `alg`
  `HS256`.
- `supabase gen signing-key --help`, CLI 3.2.0, read on 2026-11-18: "ES256 - ECDSA with P-256
  curve and SHA-256 (recommended)".
- Supabase, [JWT signing keys](https://supabase.com/docs/guides/auth/signing-keys), checked on
  2026-11-02 for STA-309: Supabase calls the shared secret "not recommended for production".
- [RFC 7518, section 3.4](https://www.rfc-editor.org/rfc/rfc7518#section-3.4): an ES256
  signature is 64 bytes.
