# ADR 0046: The Text Format of an API Key

| Field | Value |
| ----- | ----- |
| Status | Accepted by the project lead on 2026-11-26 ([STA-567 comment `afef6457`](https://linear.app/state-machine/issue/STA-567#comment-afef6457)) |
| Date | 2026-11-26 |
| Deciders | The project lead |
| Ticket | [STA-567](https://linear.app/state-machine/issue/STA-567) |

## Context

STA-567 adds `ApiKey` in `api/src/shared/api_key/`. A client sends the key in
`Authorization: Bearer` to call `/v1`. The ticket fixes the outline of the format:

- The prefix is `ak_live_` in production and `ak_test_` in development.
- The secret is 32 bytes from the random source of the operating system, in base62, 43
  characters.
- The checksum is a CRC32 of the prefix and the secret, in base62, 6 characters.
- The API stores only the SHA-256 digest of the full key.

The outline leaves the exact bytes open: the base62 alphabet, how the bytes become digits, which
bytes the CRC32 reads, and how the checksum joins the secret. Each of these choices changes the
text of every key.

A key that a client holds must keep working. A change of the format after the first key is issued
breaks every issued key: the checksum check rejects it, or its digest no longer matches. The
choice binds the access middleware (STA-581), the create endpoint (STA-582), and any secret
scanner that later matches the pattern.

## Decision

1. **A key is `ak_<environment>_<secret>_<checksum>`, 58 ASCII bytes.** `<environment>` is the
   stored name of `KeyEnvironment` from ADR 0045: `live` or `test`.
2. **The base62 alphabet is `0-9A-Za-z`, in ASCII order.** Digit 0 is `0`, digit 61 is `z`.
3. **The secret is the 32 bytes as one big-endian number, written in base62 with exactly 43
   digits, padded on the left with `0`.** 43 is the fewest digits that hold every 32-byte value:
   62⁴³ > 2²⁵⁶ > 62⁴².
4. **The checksum is CRC-32/ISO-HDLC of the ASCII bytes of `ak_<environment>_<secret>`, written
   in base62 with exactly 6 digits, padded on the left with `0`.** This CRC-32 is the one of zlib
   and of the crate `crc32fast`. The CRC reads the prefix, so a key of one environment fails the
   checksum in the other. It does not read the `_` before the checksum. 6 is the fewest digits that
   hold every CRC32: 62⁶ > 2³² > 62⁵.
5. **The digest is the SHA-256 of the 58 bytes of the full key.**

## Options Considered

No option was measured. The digit counts follow from the arithmetic above. The test key of the
bytes 0 to 31 is `ak_live_003aUlTJC7tjlCTQj2uNU3MFagCXG9LRKRcwGkBIDlf_0gEjkH`. Python's `zlib.crc32`
and `int.from_bytes` give the same text as the Rust code on 2026-11-26.

| Option | A key relabeled to the other environment | Rust code | Matches a reference outside Rust |
| ------ | ---------------------------------------- | --------- | -------------------------------- |
| **A (chosen):** CRC of prefix and secret | Fails the checksum, without a database call | `crc32fast::hash` | Yes, `zlib.crc32` |
| B: CRC of the secret only | Passes the checksum. It fails only at the database lookup | Same | Yes |
| C: no checksum | Every invented key reaches the database | None | — |

| Option | Order of the digits | Note |
| ------ | ------------------- | ---- |
| **A (chosen):** `0-9A-Za-z` | ASCII order equals numeric order | A sort of the text sorts the numbers |
| B: `0-9a-zA-Z` | Not ASCII order | No fact found that favors it |

- The CRC of A covers the prefix, which is the outline of the ticket.
- Fixed widths make every key 58 bytes, so `ApiKey::parse` checks the length before it reads a
  character.

## Consequences

- `ApiKey::parse` rejects a mistyped key, a key of the other environment, and a non-key string,
  such as a login token, without I/O.
- A change of any part of this format needs a second format that `ApiKey::parse` accepts next to
  this one, until every key of this format is revoked.
- The decision does not cover the display prefix (`ak_live_` and 4 characters of the secret), the
  number of keys per account, or the rotation of keys. The create and list endpoints decide them.
- A secret scanner, such as the one of GitHub, can match the pattern
  `ak_(live|test)_[0-9A-Za-z]{43}_[0-9A-Za-z]{6}`. Joining such a program is not part of this
  decision.

## Open Questions

None.

## Sources

- [STA-567](https://linear.app/state-machine/issue/STA-567), Description and Definition of Done,
  read on 2026-11-26.
- [ADR 0045](0045-stored-names-of-access-enums.md), the stored names of `KeyEnvironment`.
- `python3 -c 'print(62**43 > 2**256, 62**42 >= 2**256, 62**6 > 2**32, 62**5 >= 2**32)'` prints
  `True False True False`, run on 2026-11-26.
