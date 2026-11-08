//! # Notes Parser
//!
//! Maps the rows of one filing of the SEC Financial Statement and Notes data set into
//! [`AsIsFact`] values and one [`Filing`].
//!
//! The transform opens the Notes ZIP, splits its tables by `adsh`, and hands this parser the rows of
//! one filing as [`NotesFiling`]. The parser reads no file. Its rules make a Notes row equal to the
//! fact that the instance document of the filing reports, see `xbrl/DESIGN.md`, "Decisions for the
//! Transform Design".
//!
//! ## Modules
//!
//! - [`row`]: The row types of the Notes tables, and the map of each column into a core type.

use std::collections::{BTreeSet, HashMap, HashSet};

use self::row::{DimRow, NumRow, SubRow, TagRow};
use crate::core::concept::{ConceptName, NamespaceFamily};
use crate::core::fact::{AsIsFact, Context, FactKey, FilingSource, TaxonomyVersion};
use crate::core::filing::Filing;
use crate::core::finding::Finding;
use crate::core::invalid_value::{InvalidValue, InvalidValueReason};
use crate::core::taxonomy::registry::Registry;
use crate::error::parsing::ParseErrorKind;

pub mod row;

/// The rows of one filing of a Notes file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotesFiling {
    /// The `sub` row of the filing.
    pub sub: SubRow,
    /// The `num` rows of the filing.
    pub num: Vec<NumRow>,
    /// The `dim` rows that the `num` rows name through `dimh`. The row of `0x00000000`, which has no
    /// pairs, is one of them.
    pub dim: Vec<DimRow>,
    /// The `tag` rows that the `num` rows name through `tag` and `version`.
    pub tag: Vec<TagRow>,
}

/// The as-is facts, the filing, and the findings of one Notes filing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotesFacts {
    /// One fact per `num` row, in the order of the rows, without the rows that repeat a fact.
    pub facts: Vec<AsIsFact>,
    /// The filing, from the `sub` row.
    pub filing: Filing,
    /// One finding per row the parser dropped, then one per undeclared family.
    pub findings: Vec<Finding>,
}

/// Parses the rows of one Notes filing into as-is facts, one filing, and findings.
///
/// - The registry splits `num.version` into a family and a taxonomy version. A `version` equal to
///   the accession number marks an extension: its family and version are the filing ID. Any other
///   `version` keeps its fact with the full `version` as family and version, and gives one
///   [`Finding::UndeclaredFamily`].
/// - A row that breaks a rule of a core type gives a [`Finding::UnmappedRow`] and no fact.
/// - Rows equal in value, `decimals`, and every key field are one fact. `copy_rank` numbers the
///   other rows of one key in `iprx` order, then in the order of the rows.
///
/// # Errors
///
/// Returns [`ParseErrorKind::InvalidSubmissionRow`] if the `sub` row breaks a rule of a core type.
pub fn parse(rows: &NotesFiling, registry: &Registry<'_>) -> Result<NotesFacts, ParseErrorKind> {
    let filing = rows
        .sub
        .filing()
        .map_err(|source| ParseErrorKind::InvalidSubmissionRow {
            adsh: rows.sub.adsh.clone(),
            source,
        })?;
    let parser = Parser {
        filing: &filing,
        adsh: &rows.sub.adsh,
        registry,
        dims: rows
            .dim
            .iter()
            .map(|dim| (dim.dimh.as_str(), dim))
            .collect(),
        per_share: rows
            .tag
            .iter()
            .filter(|tag| tag.is_per_share())
            .map(|tag| (tag.tag.as_str(), tag.version.as_str()))
            .collect(),
    };
    let mut findings = Vec::new();
    let mut candidates = Vec::new();
    for row in &rows.num {
        match parser.candidate(row) {
            Ok(candidate) => candidates.push(candidate),
            Err(reason) => findings.push(parser.unmapped(row, reason)),
        }
    }
    let facts = number_copies(candidates, &parser, &mut findings);
    findings.extend(parser.undeclared_families(&facts));
    Ok(NotesFacts {
        facts,
        filing,
        findings,
    })
}

struct Parser<'a> {
    filing: &'a Filing,
    adsh: &'a str,
    registry: &'a Registry<'a>,
    dims: HashMap<&'a str, &'a DimRow>,
    per_share: HashSet<(&'a str, &'a str)>,
}

struct Candidate<'a> {
    row: &'a NumRow,
    copy_index: u32,
    fact: AsIsFact,
}

impl Parser<'_> {
    fn candidate<'r>(&self, row: &'r NumRow) -> Result<Candidate<'r>, InvalidValue> {
        let (concept, taxonomy_version) = self.concept(row)?;
        let per_share = self
            .per_share
            .contains(&(row.tag.as_str(), row.version.as_str()));
        let fact = AsIsFact {
            source: FilingSource {
                filing: self.filing.id.clone(),
                filed: self.filing.filed,
                accepted: self.filing.accepted,
            },
            context: Context {
                entity: self.filing.entity.clone(),
                period: row.period()?,
                dimensions: row.dimensions(&self.dims)?,
            },
            concept,
            taxonomy_version: Some(taxonomy_version),
            unit: Some(row.unit(per_share)?),
            value: row.value()?,
            decimals: row.decimals()?,
            copy_rank: 0,
            footnotes: row.footnotes(),
        };
        Ok(Candidate {
            row,
            copy_index: row.copy_index()?,
            fact,
        })
    }

    fn concept(&self, row: &NumRow) -> Result<(ConceptName, TaxonomyVersion), InvalidValue> {
        let (family, version) = match self.registry.split_notes_version(&row.version) {
            Some(split) => split,
            None if row.version == self.adsh => {
                let filing_id = self.filing.id.to_string();
                (
                    NamespaceFamily::new(filing_id.clone())?,
                    TaxonomyVersion::new(filing_id)?,
                )
            }
            None => (
                NamespaceFamily::new(row.version.clone())?,
                TaxonomyVersion::new(row.version.clone())?,
            ),
        };
        Ok((ConceptName::new(family, row.tag.clone())?, version))
    }

    fn unmapped(&self, row: &NumRow, reason: InvalidValue) -> Finding {
        Finding::UnmappedRow {
            filing: self.filing.id.clone(),
            row: row.to_string(),
            reason,
        }
    }

    fn undeclared_families(&self, facts: &[AsIsFact]) -> Vec<Finding> {
        let extension = self.filing.id.to_string();
        let families: BTreeSet<&NamespaceFamily> = facts
            .iter()
            .map(|fact| fact.concept.family())
            .filter(|family| {
                family.as_str() != extension && self.registry.declaration(family).is_none()
            })
            .collect();
        families
            .into_iter()
            .map(|family| Finding::UndeclaredFamily {
                filing: self.filing.id.clone(),
                family: family.clone(),
            })
            .collect()
    }
}

/// Drops the rows that repeat a fact, and numbers the copies of each key in `iprx` order.
fn number_copies(
    mut candidates: Vec<Candidate<'_>>,
    parser: &Parser<'_>,
    findings: &mut Vec<Finding>,
) -> Vec<AsIsFact> {
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by_key(|&index| candidates[index].copy_index);
    let mut copies: HashMap<FactKey, Vec<usize>> = HashMap::new();
    let mut kept = vec![false; candidates.len()];
    for index in order {
        let fact = &candidates[index].fact;
        let earlier = copies.entry(fact.key()).or_default();
        if earlier.iter().any(|&other| {
            candidates[other].fact.value == fact.value
                && candidates[other].fact.decimals == fact.decimals
        }) {
            continue;
        }
        match u16::try_from(earlier.len()) {
            Ok(rank) => {
                earlier.push(index);
                candidates[index].fact.copy_rank = rank;
                kept[index] = true;
            }
            Err(_) => findings.push(parser.unmapped(
                candidates[index].row,
                InvalidValue::new(
                    COPY_RANK_TYPE_NAME,
                    InvalidValueReason::OutOfRange,
                    candidates[index].row.iprx.clone(),
                ),
            )),
        }
    }
    candidates
        .into_iter()
        .zip(kept)
        .filter_map(|(candidate, kept)| kept.then_some(candidate.fact))
        .collect()
}

/// The type name that [`InvalidValue`] names when a key has more copies than `copy_rank` holds.
const COPY_RANK_TYPE_NAME: &str = "copy_rank";

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::concept::NamespaceFamily;
    use crate::core::fact::FactValue;
    use crate::core::filing::FilingId;
    use crate::core::taxonomy::fake::{FakeTaxonomy, declaration, plugin};

    const ADSH: &str = "0000000001-26-000001";

    fn us_gaap() -> FakeTaxonomy {
        plugin(
            "us-gaap/2026",
            vec![declaration(
                "us-gaap",
                "http://fasb.org/us-gaap/{release}",
                true,
            )],
        )
    }

    fn num_row(tag: &str, version: &str, iprx: &str, value: &str) -> NumRow {
        NumRow {
            tag: tag.to_owned(),
            version: version.to_owned(),
            ddate: "20260930".to_owned(),
            qtrs: "1".to_owned(),
            uom: "USD".to_owned(),
            dimh: "0x00000000".to_owned(),
            iprx: iprx.to_owned(),
            value: value.to_owned(),
            durp: "0.0".to_owned(),
            datp: "0.0".to_owned(),
            dcml: "-6".to_owned(),
            ..NumRow::default()
        }
    }

    fn notes_filing(num: Vec<NumRow>) -> NotesFiling {
        NotesFiling {
            sub: SubRow {
                adsh: ADSH.to_owned(),
                cik: "1".to_owned(),
                form: "10-Q".to_owned(),
                filed: "20261029".to_owned(),
                ..SubRow::default()
            },
            num,
            dim: vec![DimRow {
                dimh: "0x00000000".to_owned(),
                segments: String::new(),
                segt: "0".to_owned(),
            }],
            tag: vec![TagRow {
                tag: "EarningsPerShareBasic".to_owned(),
                version: "us-gaap/2026".to_owned(),
                datatype: "perShare".to_owned(),
            }],
        }
    }

    fn filing_id() -> FilingId {
        FilingId::new("sec", ADSH)
            .expect("Given a hardcoded source and ID, the creation should always succeed")
    }

    fn family(text: &str) -> NamespaceFamily {
        NamespaceFamily::new(text).expect(
            "Given a hardcoded family without whitespace, the creation should always succeed",
        )
    }

    fn registry(plugin: &FakeTaxonomy) -> Registry<'_> {
        Registry::new(vec![plugin])
            .expect("Given one plug-in with one family, the creation should always succeed")
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_notes_filing() {
        implements_auto_traits::<NotesFiling>();
    }

    #[test]
    const fn should_implement_auto_traits_for_notes_facts() {
        implements_auto_traits::<NotesFacts>();
    }

    #[test]
    fn should_number_copies_in_iprx_order_when_one_key_has_two_values() {
        let num = vec![
            num_row("Revenues", "us-gaap/2026", "1", "200.0000"),
            num_row("Revenues", "us-gaap/2026", "0", "100.0000"),
        ];
        let rows = notes_filing(num);
        let plugin = us_gaap();
        let registry = registry(&plugin);

        let expected_result = vec![1, 0];

        let result: Vec<u16> = parse(&rows, &registry)
            .expect("Given a valid sub row, the parse should always succeed")
            .facts
            .iter()
            .map(|fact| fact.copy_rank)
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_one_fact_when_two_rows_are_equal_in_value_and_decimals() {
        let num = vec![
            num_row("Revenues", "us-gaap/2026", "0", "100.0000"),
            num_row("Revenues", "us-gaap/2026", "1", "100.0000"),
        ];
        let rows = notes_filing(num);
        let plugin = us_gaap();
        let registry = registry(&plugin);

        let expected_result = 1;

        let result = parse(&rows, &registry)
            .expect("Given a valid sub row, the parse should always succeed")
            .facts
            .len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_a_nil_fact_when_the_value_is_empty() {
        let num = vec![NumRow {
            dcml: "-32768".to_owned(),
            ..num_row("CommitmentsAndContingencies", "us-gaap/2026", "0", "")
        }];
        let rows = notes_filing(num);
        let plugin = us_gaap();
        let registry = registry(&plugin);

        let expected_result = vec![(FactValue::Nil, None)];

        let result: Vec<_> = parse(&rows, &registry)
            .expect("Given a valid sub row, the parse should always succeed")
            .facts
            .into_iter()
            .map(|fact| (fact.value, fact.decimals))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_restore_shares_when_the_tag_row_names_a_per_share_tag() {
        let num = vec![num_row(
            "EarningsPerShareBasic",
            "us-gaap/2026",
            "0",
            "1.9500",
        )];
        let rows = notes_filing(num);
        let plugin = us_gaap();
        let registry = registry(&plugin);

        let expected_result = vec!["USD/shares".to_owned()];

        let result: Vec<String> = parse(&rows, &registry)
            .expect("Given a valid sub row, the parse should always succeed")
            .facts
            .iter()
            .filter_map(|fact| fact.unit.as_ref().map(ToString::to_string))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_split_the_version_when_the_registry_declares_the_family() {
        let num = vec![num_row("Revenues", "us-gaap/2026", "0", "100.0000")];
        let rows = notes_filing(num);
        let plugin = us_gaap();
        let registry = registry(&plugin);

        let expected_result = vec![(
            "us-gaap:Revenues".to_owned(),
            Some("us-gaap/2026".to_owned()),
        )];

        let result: Vec<(String, Option<String>)> = parse(&rows, &registry)
            .expect("Given a valid sub row, the parse should always succeed")
            .facts
            .iter()
            .map(|fact| {
                let version = fact.taxonomy_version.as_ref().map(ToString::to_string);
                (fact.concept.to_string(), version)
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_the_filing_id_as_family_and_version_when_the_version_is_the_accession_number() {
        let num = vec![num_row("CovenantRatio", ADSH, "0", "3.5000")];
        let rows = notes_filing(num);
        let plugin = us_gaap();
        let registry = registry(&plugin);

        let expected_result = vec![(
            "sec:0000000001-26-000001:CovenantRatio".to_owned(),
            Some("sec:0000000001-26-000001".to_owned()),
        )];

        let result: Vec<(String, Option<String>)> = parse(&rows, &registry)
            .expect("Given a valid sub row, the parse should always succeed")
            .facts
            .iter()
            .map(|fact| {
                let version = fact.taxonomy_version.as_ref().map(ToString::to_string);
                (fact.concept.to_string(), version)
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_the_facts_and_give_one_finding_when_a_family_is_undeclared() {
        let num = vec![
            num_row("Exposure", "xyz/2027", "0", "1.0000"),
            num_row("Limit", "xyz/2027", "0", "2.0000"),
        ];
        let rows = notes_filing(num);
        let plugin = us_gaap();
        let registry = registry(&plugin);

        let expected_result = (
            2,
            vec![Finding::UndeclaredFamily {
                filing: filing_id(),
                family: family("xyz/2027"),
            }],
        );

        let notes_facts = parse(&rows, &registry)
            .expect("Given a valid sub row, the parse should always succeed");
        let result = (notes_facts.facts.len(), notes_facts.findings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_the_row_and_give_a_finding_when_ddate_is_not_a_date() {
        let num = vec![NumRow {
            ddate: "2026-09-30".to_owned(),
            ..num_row("Revenues", "us-gaap/2026", "0", "100.0000")
        }];
        let rows = notes_filing(num);
        let plugin = us_gaap();
        let registry = registry(&plugin);

        let expected_result = (
            0,
            vec![Finding::UnmappedRow {
                filing: filing_id(),
                row: "num.tsv Revenues us-gaap/2026 2026-09-30 1 USD 0x00000000 0".to_owned(),
                reason: InvalidValue::new(
                    "NumRow",
                    InvalidValueReason::NotADate("%Y%m%d"),
                    "2026-09-30",
                ),
            }],
        );

        let notes_facts = parse(&rows, &registry)
            .expect("Given a valid sub row, the parse should always succeed");
        let result = (notes_facts.facts.len(), notes_facts.findings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_the_filing_when_the_sub_row_has_no_valid_cik() {
        let mut rows = notes_filing(vec![]);
        rows.sub.cik = "CIK1".to_owned();
        let plugin = us_gaap();
        let registry = registry(&plugin);

        let expected_result = "[InvalidSubmissionRow] Invalid sub row of the Notes filing '0000000001-26-000001', Caused by: [InvalidValue] Invalid 'SubRow', Reason: 'The value is not a number', Input: 'CIK1'";

        let result = parse(&rows, &registry)
            .expect_err("Given a CIK with letters, the parse should always fail")
            .to_string();

        assert_eq!(result, expected_result);
    }
}
