# ADR 0010: The Stores Keep Only Identifier Schemes That Need No Paid Licence

| Field | Value |
| ----- | ----- |
| Status | Accepted by the lead on 2026-11-14 ([STA-501 comment `af013d35`](https://linear.app/state-machine/issue/STA-501/design-identifiers-and-classifications-of-companies-securities-and#comment-af013d35)). Amendment 1: Accepted by the project lead on 2026-11-15 ([STA-519 comment `9c781b3a`](https://linear.app/state-machine/issue/STA-519#comment-9c781b3a)). |
| Date | 2026-11-14 |
| Deciders | The project lead |
| Ticket | [STA-501](https://linear.app/state-machine/issue/STA-501) |

## Context

Some identifier schemes need a paid licence to store or to serve. The lead raised this on
2026-10-31 in the fog log of the Linear project MVP Data Pt. 2, "Identifier licences". The fog log
proposes CIK, LEI, ticker, and FIGI, and no CUSIP or ISIN, with every licence unverified.

The choice binds each loader of identifiers, the `/v1` API, and the price store. If a store keeps
a licensed code, a reversal deletes stored data.

## Decision

1. **The stores keep only the schemes in the table "Allowed" below.** A new scheme joins the table
   after a licence check, with the date and the source of the check. A scheme marked "Unverified"
   is allowed for design only. No loader of it ships before a check of the primary page confirms
   the licence.
2. **No store keeps a CUSIP, an ISIN, or a GICS code.** A US ISIN holds the CUSIP. The licence of
   the other ISINs depends on the national numbering agency, and nobody checked it, so the rule
   covers every ISIN.
3. **Each licence carries the date and the source of its check.** A licence that no primary page
   confirms stays marked "unverified".

### Allowed

| Scheme | Licence found | Checked |
| ------ | ------------- | ------- |
| `cik`, `us_ticker`, `sic` | The SEC says that anyone can access and download EDGAR data for free. No SEC page names a licence. A third party calls EDGAR content a work of the US government, in the public domain under 17 USC §105 | Checked 2026-11-15. The SEC says that information on sec.gov "is considered public information and may be copied or further distributed by users of the web site without the SEC's permission". [SEC, privacy information](https://www.sec.gov/about/privacy-information), "Website Dissemination" |
| `lei` | CC0, as the AWS Open Data registry lists the GLEIF data | Checked 2026-11-15. GLEIF says that "the data available through the Access Service are provided under the CC0 licence". [GLEIF, LEI data terms of use](https://www.gleif.org/en/meta/lei-data-terms-of-use), "II. Access Service" |
| `figi`, `figi_share_class` | The OpenFIGI terms of service are a public domain dedication. Bloomberg describes FIGI as offered under the MIT licence, with no restriction on use or redistribution | Checked 2026-11-15. Bloomberg "dedicates FIGI Identifiers to the public domain", and allows their redistribution to customers. The terms name no MIT licence. [OpenFIGI terms of service](https://www.openfigi.com/docs/terms-of-service), last updated 2018-11-27, "1. Public Domain Dedication" |
| MIC (ISO 10383) | The ISO 10383 registration authority publishes the list for free download | Checked 2026-11-15. The terms add a limit on copies. Amendment 1 covers it. [ISO 10383 terms of use](https://www.iso20022.org/sites/default/files/2020-02/ISO10383_Terms_of_use.pdf), version October 2016 |

### Not Kept

| Scheme | Licence found | Checked |
| ------ | ------------- | ------- |
| `cusip`, `isin` | The terms of the ANNA DSB say that any use outside clearing and settlement needs a licence from CUSIP Global Services, with a fee based on usage. A US ISIN holds the CUSIP | Checked 2026-11-15 for CUSIPs and US ISINs. The CGS terms on the ANNA DSB page say: "Any use by Subscriber outside of the clearing and settlement of transactions requires a license from CGS, along with an associated fee based on usage." [ANNA DSB, third party data terms](https://www.anna-dsb.com/dsb-third-party-data/). CGS says that its CUSIP database holds "CGS ISINs". [CGS, license fees](https://www.cusip.com/services/license-fees.html). The ISINs of other numbering agencies are unverified |
| `gics` | MSCI and S&P own GICS. A user agreement governs its use, and GICS Direct sells the codes per company | Unverified. Web search, 2026-11-14 |

## Amendment 1: No Endpoint Serves the MIC List

Date: 2026-11-15.

**Reason.** The ISO 10383 terms of use grant "a non-exclusive right to access and use the
information on this website excluding the right to reproduce all or substantial part of this
website for the purpose of making such reproductions available to third parties". This ADR reads
"this website" as the MIC list that the website publishes. Decision 1 allows MIC with no limit on
copies. The terms give no number for "substantial part".

**Decision.** MIC stays in the table "Allowed". A store can keep the whole ISO 10383 list as the
reference data of `Exchange`, because the limit covers only copies for third parties. `/v1` serves
a MIC only as an attribute of the exchange that a listing names. No endpoint serves the ISO 10383
list, or a substantial part of it, as a list of its own.

**Consequences.**

- The repository maps 22 EDGAR exchange codes to 21 MICs in
  `sec/src/lib/shared/edgar_exchange_code/mod.rs`, read on 2026-11-15. The
  [ISO 10383 list](https://www.iso20022.org/sites/default/files/ISO10383_MIC/ISO10383_MIC.csv),
  downloaded on 2026-11-15, holds 2,893 MICs, of which 2,304 have status `ACTIVE`. The project
  lead decides whether 21 of 2,893 is a substantial part.
- An endpoint that lists exchanges with their MICs needs a check of its size against the list. The
  project lead decides the size.
- This amendment does not cover the other columns of the MIC list, such as the operating MIC or the
  country.

## Options Considered

No option was measured.

| Option | Cost | Risk |
| ------ | ---- | ---- |
| **A (chosen):** keep only schemes with no paid licence | None | A user who searches by ISIN or CUSIP finds nothing |
| B: keep CUSIP and ISIN for internal use only | A CGS licence, with a waiver below 500 identifiers a year (WatersTechnology, unverified) | Internal use still needs a signed licence |
| C: keep every scheme and decide later | None today | A reversal deletes stored codes |

- A needs no contract and no fee. A FIGI names the same securities and listings for free.
- B and C store codes before a person checks the licence.

## Consequences

- The security level takes its keys from FIGI, not from ISIN or CUSIP.
- A loader that reads a file with ISINs or CUSIPs drops them before it writes silver or gold.
- `/v1` returns no CUSIP, no ISIN, and no GICS sector. Sectors come from SIC.

## Open Questions

- Whether bronze can keep a raw file that holds ISINs, such as the GLEIF and ANNA map from LEI to
  ISIN, which the FIGI path of the fog log reads. The owner decides with a lawyer. Open on
  2026-11-15.
- Whether a loader can hold an ISIN in memory as a temporary key to reach a FIGI. The owner decides
  with a lawyer. Open on 2026-11-15.
- Whether the licence of an ISIN from a numbering agency outside the US allows storing it. The ANNA
  DSB third party data page, read on 2026-11-15, covers only CUSIP, SEDOL, and FIGI data in the
  DSB services. A check of the terms of each numbering agency answers it.
- Whether the repository can hold MICs in source code or test fixtures, and how many. Amendment 1
  covers stores and `/v1` only. The repository holds 21 of 2,893 MICs on 2026-11-15. The project
  lead decides.
- This decision reopens if a licence above changes, or if a check of a primary page contradicts the
  table.

## Sources

- The fog log of the Linear project MVP Data Pt. 2, "Identifier licences", read on 2026-11-14.
- [Linear document "STA-519 Findings: Licences of the Identifier Schemes"](https://linear.app/state-machine/document/sta-519-findings-licences-of-the-identifier-schemes-e3fed0a56a6d):
  the quotes of each primary page below, and the answers of the owner, 2026-11-15.
- SEC, [privacy information](https://www.sec.gov/about/privacy-information), "Website
  Dissemination", last reviewed 2023-11-29, read on 2026-11-15.
- SEC, [Accessing EDGAR Data](https://www.sec.gov/search-filings/edgar-search-assistance/accessing-edgar-data):
  "Anyone can access and download this information for free", read on 2026-11-15. The URL of the
  web search below redirects here.
- GLEIF, [LEI data terms of use](https://www.gleif.org/en/meta/lei-data-terms-of-use), read on
  2026-11-15.
- [OpenFIGI terms of service](https://www.openfigi.com/docs/terms-of-service), last updated
  2018-11-27, read on 2026-11-15.
- [ISO 10383 terms of use](https://www.iso20022.org/sites/default/files/2020-02/ISO10383_Terms_of_use.pdf),
  version October 2016, read on 2026-11-15.
- CUSIP Global Services, [license fees](https://www.cusip.com/services/license-fees.html), read on
  2026-11-15.
- [ANNA DSB, third party data terms](https://www.anna-dsb.com/dsb-third-party-data/), read on
  2026-11-15.
- [ISO 10383 MIC list](https://www.iso20022.org/sites/default/files/ISO10383_MIC/ISO10383_MIC.csv),
  downloaded on 2026-11-15: 2,893 rows, 2,304 with status `ACTIVE`.
- SEC, [Accessing EDGAR Data](https://www.sec.gov/os/accessing-edgar-data), web search result,
  2026-11-14.
- [Cybersyn, SEC filings](https://docs.cybersyn.com/public-domain/public-domain-data-sources/sec-filings):
  the 17 USC §105 claim, web search result, 2026-11-14.
- [AWS Open Data registry, LEI](https://registry.opendata.aws/lei/): CC0, web search result,
  2026-11-14.
- [Bloomberg press release](https://www.bloomberg.com/company/press/bloomberg-launches-online-request-utility-and-new-mapping-tools-for-the-financial-instrument-global-identifier-figi):
  the MIT claim, web search result, 2026-11-14.
- [ISO 20022, market identifier codes](https://www.iso20022.org/market-identifier-codes), web
  search result, 2026-11-14.
- [WatersTechnology, ISIN licensing](https://www.waterstechnology.com/inside-reference-data/news/1634255/understanding-isin-licensing-practices):
  the waiver of option B, web search result, 2026-11-14.
- [MSCI, GICS](https://msci.com/indexes/index-resources/gics) and
  [GICS service third party terms](https://www.fisglobal.com/-/media/fisglobal/files/PDF/policy/third-party-terms/GICS-Service-Third-Party-Data-Terms.pdf),
  web search results, 2026-11-14.
