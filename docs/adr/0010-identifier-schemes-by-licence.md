# ADR 0010: The Stores Keep Only Identifier Schemes That Need No Paid Licence

| Field | Value |
| ----- | ----- |
| Status | Proposed. The project lead confirms. |
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
   after a licence check, with the date and the source of the check.
2. **No store keeps a CUSIP, an ISIN, or a GICS code.** A US ISIN holds the CUSIP, so this covers
   every ISIN.
3. **Each licence carries the date and the source of its check.** A licence that no primary page
   confirms stays marked "unverified".

### Allowed

| Scheme | Licence found | Checked |
| ------ | ------------- | ------- |
| `cik`, `us_ticker`, `sic` | The SEC says that anyone can access and download EDGAR data for free. No SEC page names a licence. A third party calls EDGAR content a work of the US government, in the public domain under 17 USC §105 | Unverified. Web search, 2026-11-14 |
| `lei` | CC0, as the AWS Open Data registry lists the GLEIF data | Unverified. The GLEIF terms of use page was not reachable from the container, 2026-11-14 |
| `figi`, `figi_share_class` | The OpenFIGI terms of service are a public domain dedication. Bloomberg describes FIGI as offered under the MIT licence, with no restriction on use or redistribution | Unverified. The OpenFIGI page was not reachable from the container, 2026-11-14 |
| MIC (ISO 10383) | The ISO 10383 registration authority publishes the list for free download | Unverified. The terms of use PDF was not reachable from the container, 2026-11-14 |

### Not Kept

| Scheme | Licence found | Checked |
| ------ | ------------- | ------- |
| `cusip`, `isin` | The terms of the ANNA DSB say that any use outside clearing and settlement needs a licence from CUSIP Global Services, with a fee based on usage. A US ISIN holds the CUSIP | Unverified. Web search, 2026-11-14 |
| `gics` | MSCI and S&P own GICS. A user agreement governs its use, and GICS Direct sells the codes per company | Unverified. Web search, 2026-11-14 |

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
- Each licence above needs a check of its primary page before the first loader of its scheme
  ships.

## Open Questions

- Whether bronze can keep a raw file that holds ISINs, such as the GLEIF and ANNA map from LEI to
  ISIN, which the FIGI path of the fog log reads. The owner decides with a lawyer.
- Whether a loader can hold an ISIN in memory as a temporary key to reach a FIGI. The owner decides
  with a lawyer.
- This decision reopens if a licence above changes, or if a check of a primary page contradicts the
  table.

## Sources

- The fog log of the Linear project MVP Data Pt. 2, "Identifier licences", read on 2026-11-14.
- SEC, [Accessing EDGAR Data](https://www.sec.gov/os/accessing-edgar-data), web search result,
  2026-11-14.
- [Cybersyn, SEC filings](https://docs.cybersyn.com/public-domain/public-domain-data-sources/sec-filings):
  the 17 USC §105 claim, web search result, 2026-11-14.
- [AWS Open Data registry, LEI](https://registry.opendata.aws/lei/): CC0, web search result,
  2026-11-14.
- [OpenFIGI terms of service](https://openfigi.com/docs/terms-of-service) and
  [Bloomberg press release](https://www.bloomberg.com/company/press/bloomberg-launches-online-request-utility-and-new-mapping-tools-for-the-financial-instrument-global-identifier-figi),
  web search results, 2026-11-14.
- [ISO 20022, market identifier codes](https://www.iso20022.org/market-identifier-codes), web
  search result, 2026-11-14.
- [ANNA DSB, third party data terms](https://www.anna-dsb.com/dsb-third-party-data/) and
  [WatersTechnology, ISIN licensing](https://www.waterstechnology.com/inside-reference-data/news/1634255/understanding-isin-licensing-practices),
  web search results, 2026-11-14.
- [MSCI, GICS](https://msci.com/indexes/index-resources/gics) and
  [GICS service third party terms](https://www.fisglobal.com/-/media/fisglobal/files/PDF/policy/third-party-terms/GICS-Service-Third-Party-Data-Terms.pdf),
  web search results, 2026-11-14.
- The hosts `gleif.org`, `openfigi.com`, and `iso20022.org` were not reachable from the container
  on 2026-11-14.
