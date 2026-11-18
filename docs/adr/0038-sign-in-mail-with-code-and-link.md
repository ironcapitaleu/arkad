# ADR 0038: The Sign-In Mail Holds a Code and a Link to the Confirm Page

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms |
| Date | 2026-11-18 |
| Deciders | The project lead |
| Ticket | [STA-312](https://linear.app/state-machine/issue/STA-312) |

## Context

The lead decided on 2026-10-31 that one page serves sign-in and sign-up, without a password. A
user enters an email, and Supabase Auth sends a mail. The content of that mail binds two places:
the mail template in `supabase/` (STA-312), and the sign-in page of the frontend (STA-316).

Supabase Auth sends one of two templates:

- **"Confirm signup"** for an email without an account.
- **"Magic link"** for an email with an account.

A **mail scanner** is a service that opens each link in an incoming mail to check it for malware.
Some company mail servers run one.

## Decision

1. **The sign-in mail holds a 6-digit code and a link to `/auth/confirm` on the frontend.** The
   link carries the token hash. The page checks the hash only after the user clicks a button.
2. **"Confirm signup" and "Magic link" use the same template**, `supabase/templates/sign_in.html`.

## Options Considered

Checked on 2026-11-18 on the local stack, Supabase CLI 3.2.0, on top of `main` at `68a45cf`.

| Option | Mail opened on another device | Mail scanner | Pages in STA-316 |
| ------ | ----------------------------- | ------------ | ---------------- |
| **A: code and link to our confirm page (chosen)** | The user types the code | Opens the page, clicks nothing, uses up nothing | Sign-in page and confirm page |
| B: code only | The user types the code | Finds no link | Sign-in page |
| C: the default link of Supabase Auth | The link signs in the other device | Opens the link and uses up the token | Sign-in page and callback |

- A and B both work when the mail opens on another device, because the code needs no browser
  state.
- A adds a sign-in with one click on the same device. The cost is the confirm page in STA-316.
- C uses up the token when a scanner opens the link, so the user gets an expired link.
- B stays the fallback if the confirm page costs too much in STA-316. Only the template changes.

## Consequences

- STA-316 builds `/auth/confirm`. It calls `verifyOtp` with `type` `email` and the `token_hash`.
- `verifyOtp` with `type` `email` accepts both the code and the token hash, and from both
  templates. The local run below checked the code from "Confirm signup" and the hash from "Magic
  link".
- The code and the link expire after 1 hour (`otp_expiry = 3600`). A user can request a new mail
  once a minute (`max_frequency = "1m"`).
- The link uses the site URL of the environment, from `SUPABASE_AUTH_SITE_URL`.
- The decision does not cover the OAuth sign-in with Google and GitHub, and it does not cover
  passkeys.

## Sources

- Local run on 2026-11-18, mails read in Mailpit. A second run gave each template its own subject.
  A new email got the subject of "Confirm signup", and a known email got the subject of "Magic
  link":
  - `POST /auth/v1/otp` for a new email logged `user_confirmation_requested`, the "Confirm signup"
    template. `POST /auth/v1/verify` with `type` `email` and the code returned a session.
  - The same request for that email after the sign-in logged `user_recovery_requested`, the
    "Magic link" template. `POST /auth/v1/verify` with `type` `email` and the token hash returned
    a session.
  - A second request within a minute answered `429` with `over_email_send_rate_limit`.
- Supabase, [Email templates](https://supabase.com/docs/guides/auth/auth-email-templates), checked
  on 2026-11-02 for STA-309: the mitigation for mail scanners links to a page of the site that
  verifies after a click.
- Lead decision on 2026-10-31, one passwordless page for sign-in and sign-up:
  [STA-312](https://linear.app/state-machine/issue/STA-312) description.
