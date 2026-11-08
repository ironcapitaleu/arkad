//! # Instance Parser
//!
//! Maps one XBRL instance document of an SEC filing into [`AsIsFact`] values and one [`Filing`].
//!
//! The transform reads the filing, picks its main instance document, and skips the filing-fee
//! exhibit. It hands this parser the text of the document, the filing ID, and the filed date of the
//! daily index as [`InstanceFiling`]. The parser reads no file. Its rules give a fact the same value
//! key that the Notes parser gives the row of the fact, see `xbrl/DESIGN.md`, "Decisions for the
//! Transform Design".
//!
//! ## Modules
//!
//! - `document`: The contexts, units, footnotes, and extension namespace of the document.
//! - `filing`: The map of the cover facts into the filing.

use std::cmp::Reverse;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use roxmltree::Node;

use self::document::{Document, PREFIX_SEPARATOR, XML_TRUE, name as xml_name};
use crate::adapter::edgar::as_is::{
    Candidate, number_copies, undeclared_families, unsplit_namespace,
};
use crate::core::concept::ConceptName;
use crate::core::fact::{AsIsFact, Decimals, FactValue, FilingSource, TaxonomyVersion};
use crate::core::filing::{Filing, FilingId};
use crate::core::finding::Finding;
use crate::core::invalid_value::{InvalidValue, InvalidValueReason};
use crate::core::period::Period;
use crate::core::taxonomy::registry::Registry;
use crate::core::unit::Unit;
use crate::error::parsing::ParseErrorKind;

mod document;
mod filing;

/// The namespace of the `xsi:nil` attribute.
pub const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";

/// The element and attribute names this module reads.
mod name {
    pub const NIL: &str = "nil";
    pub const UNIT_REF: &str = "unitRef";
    pub const DECIMALS: &str = "decimals";
    pub const NAMESPACE: &str = "namespace";
}

/// The character between the fields of a fact in a finding.
const FIELD_SEPARATOR: &str = " ";

/// One instance document of a filing, with the attributes of the filing that the document lacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceFiling<'a> {
    /// The XML text of the main instance document.
    pub document: &'a str,
    /// The ID of the filing, for example `sec:0001628280-26-068246`.
    pub id: FilingId,
    /// The filing date of the daily index.
    pub filed: NaiveDate,
}

impl InstanceFiling<'_> {
    /// The type name that [`InvalidValue`] names when a cover fact breaks a rule.
    const TYPE_NAME: &'static str = "InstanceFiling";
}

/// The type name that [`InvalidValue`] names when a fact element breaks a rule.
const FACT_TYPE_NAME: &str = "InstanceFact";

/// The as-is facts, the filing, and the findings of one instance document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceFacts {
    /// One fact per fact element, in document order, without the elements that repeat a fact.
    pub facts: Vec<AsIsFact>,
    /// The filing, from the input and the cover facts.
    pub filing: Filing,
    /// One finding per fact element the parser dropped, then one per undeclared family.
    pub findings: Vec<Finding>,
}

/// Parses one instance document into as-is facts, one filing, and findings.
///
/// - The registry splits the namespace URI of each fact into a family and a taxonomy version. The
///   namespace whose prefix names the schema file of the filing is the extension: its family and
///   version are the filing ID. Any other URI keeps its fact with the full URI as family and
///   version, and gives one [`Finding::UndeclaredFamily`].
/// - A fact that breaks a rule of a core type, or names a context or a unit that does, gives a
///   [`Finding::UnmappedRow`] and no fact. The row names the element and the context ID.
/// - Facts equal in value, `decimals`, and every key field are one fact. `copy_rank` numbers the
///   other facts of one key in the order of the SEC for `iprx`: higher precision first, then the
///   length nearer a full quarter, then document order. The SEC order also puts the end nearer a
///   month end first, but the facts of one key share one exact end, so that step never decides.
///
/// # Errors
///
/// Returns [`ParseErrorKind::InvalidXml`] if the text is not XML, and
/// [`ParseErrorKind::InvalidInstanceFiling`] if a cover fact is missing or breaks a rule.
pub fn parse(
    input: &InstanceFiling<'_>,
    registry: &Registry<'_>,
) -> Result<InstanceFacts, ParseErrorKind> {
    let xml =
        roxmltree::Document::parse(input.document).map_err(|error| ParseErrorKind::InvalidXml {
            filing: input.id.clone(),
            reason: error.to_string(),
        })?;
    let document = Document::read(xml.root_element());
    let filing = filing::filing(input, &document).map_err(|source| {
        ParseErrorKind::InvalidInstanceFiling {
            filing: input.id.clone(),
            source,
        }
    })?;
    let parser = Parser {
        source: FilingSource {
            filing: input.id.clone(),
            filed: input.filed,
            accepted: None,
        },
        registry,
        document: &document,
    };
    let mut findings = Vec::new();
    let mut candidates = Vec::new();
    for element in document.facts() {
        match parser.candidate(element) {
            Ok(candidate) => candidates.push(candidate),
            Err(reason) => findings.push(parser.unmapped(element, reason)),
        }
    }
    let (facts, overflow) = number_copies(candidates);
    findings.extend(
        overflow
            .into_iter()
            .map(|(element, reason)| parser.unmapped(element, reason)),
    );
    findings.extend(undeclared_families(&input.id, registry, &facts));
    Ok(InstanceFacts {
        facts,
        filing,
        findings,
    })
}

struct Parser<'p, 'a, 'input> {
    source: FilingSource,
    registry: &'p Registry<'p>,
    document: &'p Document<'a, 'input>,
}

/// The unit, the value, and the precision of a fact.
type Content = (Option<Unit>, FactValue, Option<Decimals>);

impl<'a, 'input> Parser<'_, 'a, 'input> {
    fn candidate(
        &self,
        element: Node<'a, 'input>,
    ) -> Result<Candidate<CopyOrder, Node<'a, 'input>>, InvalidValue> {
        let (concept, taxonomy_version) = self.concept(element)?;
        let context = self
            .document
            .context(element.attribute(xml_name::CONTEXT_REF).unwrap_or_default())?
            .clone();
        let (unit, value, decimals) = self.content(element)?;
        let order = CopyOrder::new(decimals, &context.period);
        let fact = AsIsFact {
            source: self.source.clone(),
            context,
            concept,
            taxonomy_version: Some(taxonomy_version),
            unit,
            value,
            decimals,
            copy_rank: 0,
            footnotes: self
                .document
                .footnotes(element.attribute(xml_name::ID).unwrap_or_default()),
        };
        Ok(Candidate {
            order,
            row: element,
            fact,
        })
    }

    fn concept(
        &self,
        element: Node<'_, '_>,
    ) -> Result<(ConceptName, TaxonomyVersion), InvalidValue> {
        let tag = element.tag_name();
        let namespace = tag.namespace().ok_or_else(|| {
            InvalidValue::new(
                FACT_TYPE_NAME,
                InvalidValueReason::MissingPart(name::NAMESPACE),
                tag.name(),
            )
        })?;
        let (family, version) = match self.registry.split_uri(namespace) {
            Some(split) => split,
            None => unsplit_namespace(
                namespace,
                self.document.is_extension(namespace),
                &self.source.filing,
            )?,
        };
        Ok((ConceptName::new(family, tag.name())?, version))
    }

    /// Returns the content: a numeric fact names a unit, and a text fact names none.
    fn content(&self, element: Node<'_, '_>) -> Result<Content, InvalidValue> {
        let is_nil = element
            .attribute((XSI_NAMESPACE, name::NIL))
            .is_some_and(|nil| XML_TRUE.contains(&nil));
        let text = document::text(element);
        let Some(unit_ref) = element.attribute(name::UNIT_REF) else {
            let value = if is_nil {
                FactValue::Nil
            } else {
                FactValue::Text(text)
            };
            return Ok((None, value, None));
        };
        let unit = self.document.unit(unit_ref)?.clone();
        let value = if is_nil {
            FactValue::Nil
        } else {
            BigDecimal::from_str(text.trim())
                .map(FactValue::Numeric)
                .map_err(|_| {
                    InvalidValue::new(FACT_TYPE_NAME, InvalidValueReason::NotANumber, text.trim())
                })?
        };
        let decimals = element
            .attribute(name::DECIMALS)
            .map(Decimals::from_str)
            .transpose()?;
        Ok((Some(unit), value, decimals))
    }

    fn unmapped(&self, element: Node<'_, '_>, reason: InvalidValue) -> Finding {
        Finding::UnmappedRow {
            filing: self.source.filing.clone(),
            row: row(element),
            reason,
        }
    }
}

/// Returns the element name as written and the context ID of a fact, as in
/// `us-gaap:Revenues c-1`.
fn row(element: Node<'_, '_>) -> String {
    let tag = element.tag_name();
    let prefix = tag
        .namespace()
        .and_then(|namespace| element.lookup_prefix(namespace));
    let name = prefix.map_or_else(
        || tag.name().to_owned(),
        |prefix| format!("{prefix}{PREFIX_SEPARATOR}{}", tag.name()),
    );
    let context = element.attribute(xml_name::CONTEXT_REF).unwrap_or_default();
    [name.as_str(), context].join(FIELD_SEPARATOR)
}

/// The place of a fact among the copies of its key, in the order of the SEC for `iprx`.
///
/// The fields compare in order: the higher precision first, then the length nearer a full quarter.
/// A fact with no precision, such as a text or a nil fact, comes last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CopyOrder {
    precision: Reverse<Option<Decimals>>,
    distance_to_full_quarter: i64,
}

impl CopyOrder {
    fn new(decimals: Option<Decimals>, period: &Period) -> Self {
        let distance_to_full_quarter = match period {
            Period::Duration(duration) => duration.distance_to_full_quarters().unwrap_or(0),
            _ => 0,
        };
        Self {
            precision: Reverse(decimals),
            distance_to_full_quarter,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::concept::NamespaceFamily;
    use crate::tests::fixtures::sample_taxonomy::{FakeTaxonomy, declaration, plugin};

    const ADSH: &str = "0000000001-26-000001";

    /// The start of a document: the namespaces, the schema, a quarter context, a unit, and the
    /// document type.
    const HEADER: &str = r#"<xbrl xmlns="http://www.xbrl.org/2003/instance"
        xmlns:link="http://www.xbrl.org/2003/linkbase" xmlns:xlink="http://www.w3.org/1999/xlink"
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xbrldi="http://xbrl.org/2006/xbrldi"
        xmlns:iso4217="http://www.xbrl.org/2003/iso4217" xmlns:us-gaap="http://fasb.org/us-gaap/2026"
        xmlns:dei="http://xbrl.sec.gov/dei/2026" xmlns:abc="http://www.abc.com/20260930"
        xmlns:xyz="http://xbrl.example.com/xyz/2027">
        <link:schemaRef xlink:href="abc-20260930.xsd" xlink:type="simple"/>
        <context id="q3"><entity><identifier scheme="http://www.sec.gov/CIK">1</identifier></entity>
            <period><startDate>2026-07-01</startDate><endDate>2026-09-30</endDate></period></context>
        <unit id="usd"><measure>iso4217:USD</measure></unit>
        <dei:DocumentType contextRef="q3">10-Q</dei:DocumentType>"#;

    /// The end of a document.
    const FOOTER: &str = "</xbrl>";

    fn registry_plugin() -> FakeTaxonomy {
        plugin(
            "us-gaap/2026",
            vec![
                declaration("us-gaap", "http://fasb.org/us-gaap/{release}", true),
                declaration("dei", "http://xbrl.sec.gov/dei/{release}", false),
            ],
        )
    }

    fn filing_id() -> FilingId {
        FilingId::new("sec", ADSH)
            .expect("Given a hardcoded source and ID, the creation should always succeed")
    }

    fn parse_body(body: &str) -> Result<InstanceFacts, ParseErrorKind> {
        let text = format!("{HEADER}{body}{FOOTER}");
        let plugin = registry_plugin();
        let registry = Registry::new(vec![&plugin])
            .expect("Given one plug-in with two families, the creation should always succeed");
        let input = InstanceFiling {
            document: &text,
            id: filing_id(),
            filed: NaiveDate::from_ymd_opt(2026, 10, 29)
                .expect("Given a hardcoded valid date, the creation should always succeed"),
        };
        parse(&input, &registry)
    }

    /// Returns the facts of the body without the document type.
    fn facts(body: &str) -> Vec<AsIsFact> {
        parse_body(body)
            .expect("Given a document with a document type, the parse should always succeed")
            .facts
            .into_iter()
            .filter(|fact| fact.concept.local_name() != "DocumentType")
            .collect()
    }

    const fn implements_auto_traits<T: Sized + Send + Sync + Unpin>() {}
    #[test]
    const fn should_implement_auto_traits_for_instance_filing() {
        implements_auto_traits::<InstanceFiling<'_>>();
    }

    #[test]
    const fn should_implement_auto_traits_for_instance_facts() {
        implements_auto_traits::<InstanceFacts>();
    }

    #[test]
    fn should_count_the_quarters_from_the_dates_when_the_context_is_a_duration() {
        let body = r#"<us-gaap:Revenues contextRef="q3" unitRef="usd" decimals="-6">100</us-gaap:Revenues>"#;

        let expected_result = vec!["2026-07-01 to 2026-09-30, 1 quarters".to_owned()];

        let result: Vec<String> = facts(body)
            .iter()
            .map(|fact| {
                let period = fact.context.period;
                format!("{period}, {} quarters", period.quarters())
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_an_instant_period_when_the_context_has_an_instant() {
        let body = r#"<context id="i"><entity><identifier scheme="http://www.sec.gov/CIK">1</identifier></entity>
            <period><instant>2026-09-30</instant></period></context>
            <us-gaap:Assets contextRef="i" unitRef="usd" decimals="-6">100</us-gaap:Assets>"#;

        let expected_result = vec![Period::Instant {
            end: NaiveDate::from_ymd_opt(2026, 9, 30)
                .expect("Given a hardcoded valid date, the creation should always succeed"),
        }];

        let result: Vec<Period> = facts(body).iter().map(|fact| fact.context.period).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_write_the_cik_with_ten_digits_when_the_identifier_is_shorter() {
        let body = r#"<us-gaap:Revenues contextRef="q3" unitRef="usd" decimals="-6">100</us-gaap:Revenues>"#;

        let expected_result = vec!["0000000001".to_owned()];

        let result: Vec<String> = facts(body)
            .iter()
            .map(|fact| fact.context.entity.id().to_owned())
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_shorten_axis_and_member_the_way_the_sec_does_when_the_member_is_explicit() {
        let body = r#"<context id="d"><entity><identifier scheme="http://www.sec.gov/CIK">1</identifier>
            <segment><xbrldi:explicitMember dimension="us-gaap:StatementBusinessSegmentsAxis">abc:FreightMember</xbrldi:explicitMember>
            <xbrldi:explicitMember dimension="us-gaap:ProductOrServiceAxis">abc:ServicesDomain</xbrldi:explicitMember></segment></entity>
            <period><instant>2026-09-30</instant></period></context>
            <us-gaap:Assets contextRef="d" unitRef="usd" decimals="-6">100</us-gaap:Assets>"#;

        let expected_result = vec![vec![
            ("BusinessSegments".to_owned(), "Freight".to_owned()),
            ("ProductOrService".to_owned(), "Services".to_owned()),
        ]];

        let result: Vec<Vec<(String, String)>> = facts(body)
            .iter()
            .map(|fact| {
                fact.context
                    .dimensions
                    .iter()
                    .map(|(axis, member)| (axis.to_owned(), member.to_owned()))
                    .collect()
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_statement_and_drop_the_prefix_of_the_content_when_the_member_is_typed() {
        let body = r#"<context id="t"><entity><identifier scheme="http://www.sec.gov/CIK">1</identifier>
            <segment><xbrldi:typedMember dimension="us-gaap:StatementOfIncomeLocationBalanceAxis">
            <us-gaap:QName.domain>us-gaap:LaborAndRelatedExpense</us-gaap:QName.domain></xbrldi:typedMember></segment></entity>
            <period><instant>2026-09-30</instant></period></context>
            <us-gaap:Assets contextRef="t" unitRef="usd" decimals="-6">100</us-gaap:Assets>"#;

        let expected_result = vec![vec![(
            "StatementOfIncomeLocationBalance".to_owned(),
            "LaborAndRelatedExpense".to_owned(),
        )]];

        let result: Vec<Vec<(String, String)>> = facts(body)
            .iter()
            .map(|fact| {
                fact.context
                    .dimensions
                    .iter()
                    .map(|(axis, member)| (axis.to_owned(), member.to_owned()))
                    .collect()
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_numerator_and_denominator_without_prefixes_when_the_unit_divides() {
        let body = r#"<unit id="eps"><divide><unitNumerator><measure>iso4217:USD</measure></unitNumerator>
            <unitDenominator><measure>xbrli:shares</measure></unitDenominator></divide></unit>
            <us-gaap:EarningsPerShareBasic contextRef="q3" unitRef="eps" decimals="2">1.95</us-gaap:EarningsPerShareBasic>"#;

        let expected_result = vec!["USD/shares".to_owned()];

        let result: Vec<String> = facts(body)
            .iter()
            .filter_map(|fact| fact.unit.as_ref().map(ToString::to_string))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_nil_value_when_the_fact_is_nil() {
        let body = r#"<us-gaap:CommitmentsAndContingencies contextRef="q3" unitRef="usd" xsi:nil="true"/>"#;

        let expected_result = vec![FactValue::Nil];

        let result: Vec<FactValue> = facts(body).into_iter().map(|fact| fact.value).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_infinite_decimals_when_the_decimals_attribute_is_inf() {
        let body = r#"<us-gaap:Revenues contextRef="q3" unitRef="usd" decimals="INF">100</us-gaap:Revenues>"#;

        let expected_result = vec![Some(Decimals::Infinite)];

        let result: Vec<Option<Decimals>> = facts(body).iter().map(|fact| fact.decimals).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_give_a_text_fact_with_no_unit_and_no_decimals_when_the_fact_names_no_unit() {
        let body = "";

        let expected_result = vec![(None, FactValue::Text("10-Q".to_owned()), None)];

        let result: Vec<Content> = parse_body(body)
            .expect("Given a document with a document type, the parse should always succeed")
            .facts
            .into_iter()
            .map(|fact| (fact.unit, fact.value, fact.decimals))
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_attach_the_footnote_text_with_collapsed_whitespace_when_a_link_names_the_fact() {
        let body = r##"<us-gaap:Revenues id="f-1" contextRef="q3" unitRef="usd" decimals="-6">100</us-gaap:Revenues>
            <link:footnoteLink xlink:type="extended" xlink:role="http://www.xbrl.org/2003/role/link">
            <link:loc xlink:type="locator" xlink:href="#f-1" xlink:label="fact"/>
            <link:footnote xlink:type="resource" xlink:label="note">Refer to <span>Note 1</span><span>2</span>,
                Related Party Transactions.</link:footnote>
            <link:footnoteArc xlink:type="arc" xlink:arcrole="http://www.xbrl.org/2003/arcrole/fact-footnote"
                xlink:from="fact" xlink:to="note"/></link:footnoteLink>"##;

        let expected_result = vec![vec![
            "Refer to Note 12, Related Party Transactions.".to_owned(),
        ]];

        let result: Vec<Vec<String>> = facts(body).into_iter().map(|fact| fact.footnotes).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_rank_the_higher_precision_first_when_one_key_has_two_values() {
        let body = r#"<us-gaap:Revenues contextRef="q3" unitRef="usd" decimals="-6">100000000</us-gaap:Revenues>
            <us-gaap:Revenues contextRef="q3" unitRef="usd" decimals="-3">100123000</us-gaap:Revenues>"#;

        let expected_result = vec![1, 0];

        let result: Vec<u16> = facts(body).iter().map(|fact| fact.copy_rank).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_rank_the_length_nearer_a_full_quarter_first_when_one_key_has_two_values() {
        let body = r#"<context id="four-months"><entity><identifier scheme="http://www.sec.gov/CIK">1</identifier></entity>
            <period><startDate>2026-06-01</startDate><endDate>2026-09-30</endDate></period></context>
            <us-gaap:Revenues contextRef="four-months" unitRef="usd" decimals="-6">200</us-gaap:Revenues>
            <us-gaap:Revenues contextRef="q3" unitRef="usd" decimals="-6">100</us-gaap:Revenues>"#;

        let expected_result = vec![1, 0];

        let result: Vec<u16> = facts(body).iter().map(|fact| fact.copy_rank).collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_keep_one_fact_when_two_facts_are_equal_in_key_value_and_decimals() {
        let body = r#"<context id="same"><entity><identifier scheme="http://www.sec.gov/CIK">1</identifier></entity>
            <period><startDate>2026-07-01</startDate><endDate>2026-09-30</endDate></period></context>
            <us-gaap:Revenues contextRef="q3" unitRef="usd" decimals="-6">100</us-gaap:Revenues>
            <us-gaap:Revenues contextRef="same" unitRef="usd" decimals="-6">100</us-gaap:Revenues>"#;

        let expected_result = 1;

        let result = facts(body).len();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_split_the_uri_when_the_registry_declares_the_family() {
        let body = r#"<us-gaap:Revenues contextRef="q3" unitRef="usd" decimals="-6">100</us-gaap:Revenues>"#;

        let expected_result = vec![(
            "us-gaap:Revenues".to_owned(),
            Some("us-gaap/2026".to_owned()),
        )];

        let result: Vec<(String, Option<String>)> = facts(body)
            .iter()
            .map(|fact| {
                let version = fact.taxonomy_version.as_ref().map(ToString::to_string);
                (fact.concept.to_string(), version)
            })
            .collect();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_use_the_filing_id_as_family_and_version_when_the_prefix_names_the_schema_file() {
        let body = r#"<abc:CovenantRatio contextRef="q3" unitRef="usd" decimals="2">3.5</abc:CovenantRatio>"#;

        let expected_result = vec![(
            "sec:0000000001-26-000001:CovenantRatio".to_owned(),
            Some("sec:0000000001-26-000001".to_owned()),
        )];

        let result: Vec<(String, Option<String>)> = facts(body)
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
        let body = r#"<xyz:Exposure contextRef="q3" unitRef="usd" decimals="0">1</xyz:Exposure>
            <xyz:Limit contextRef="q3" unitRef="usd" decimals="0">2</xyz:Limit>"#;

        let expected_result = (
            3,
            vec![Finding::UndeclaredFamily {
                filing: filing_id(),
                family: NamespaceFamily::new("http://xbrl.example.com/xyz/2027").expect(
                    "Given a hardcoded URI without whitespace, the creation should always succeed",
                ),
            }],
        );

        let parsed = parse_body(body)
            .expect("Given a document with a document type, the parse should always succeed");
        let result = (parsed.facts.len(), parsed.findings);

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_the_fact_and_name_element_and_context_when_the_context_is_missing() {
        let body = r#"<us-gaap:Revenues contextRef="c-9" unitRef="usd" decimals="-6">100</us-gaap:Revenues>"#;

        let expected_result = vec![Finding::UnmappedRow {
            filing: filing_id(),
            row: "us-gaap:Revenues c-9".to_owned(),
            reason: InvalidValue::new(
                "InstanceDocument",
                InvalidValueReason::NoMatchingElement("context"),
                "c-9",
            ),
        }];

        let result = parse_body(body)
            .expect("Given a document with a document type, the parse should always succeed")
            .findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_drop_the_fact_and_give_a_finding_when_the_value_is_not_a_number() {
        let body = r#"<us-gaap:Revenues contextRef="q3" unitRef="usd" decimals="-6">n/a</us-gaap:Revenues>"#;

        let expected_result = vec![Finding::UnmappedRow {
            filing: filing_id(),
            row: "us-gaap:Revenues q3".to_owned(),
            reason: InvalidValue::new("InstanceFact", InvalidValueReason::NotANumber, "n/a"),
        }];

        let result = parse_body(body)
            .expect("Given a document with a document type, the parse should always succeed")
            .findings;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_mark_an_amendment_when_the_amendment_flag_is_one() {
        let body = r#"<dei:AmendmentFlag contextRef="q3">1</dei:AmendmentFlag>"#;

        let expected_result = true;

        let result = parse_body(body)
            .expect("Given a document with a document type, the parse should always succeed")
            .filing
            .is_amendment;

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_the_filing_when_the_text_is_not_xml() {
        let text = "<xbrl>";
        let plugin = registry_plugin();
        let registry = Registry::new(vec![&plugin])
            .expect("Given one plug-in with two families, the creation should always succeed");
        let input = InstanceFiling {
            document: text,
            id: filing_id(),
            filed: NaiveDate::from_ymd_opt(2026, 10, 29)
                .expect("Given a hardcoded valid date, the creation should always succeed"),
        };

        let expected_result = "[InvalidXml] Failed to parse the instance document of the filing 'sec:0000000001-26-000001', Reason: 'the root node was opened but never closed'";

        let result = parse(&input, &registry)
            .expect_err("Given an element without its end tag, the parse should always fail")
            .to_string();

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_the_filing_when_the_document_has_no_document_type() {
        let text = format!(
            "{}{FOOTER}",
            HEADER.replace(
                r#"<dei:DocumentType contextRef="q3">10-Q</dei:DocumentType>"#,
                ""
            )
        );
        let plugin = registry_plugin();
        let registry = Registry::new(vec![&plugin])
            .expect("Given one plug-in with two families, the creation should always succeed");
        let input = InstanceFiling {
            document: &text,
            id: filing_id(),
            filed: NaiveDate::from_ymd_opt(2026, 10, 29)
                .expect("Given a hardcoded valid date, the creation should always succeed"),
        };

        let expected_result = "[InvalidInstanceFiling] Invalid cover facts in the instance document of the filing 'sec:0000000001-26-000001', Caused by: [InvalidValue] Invalid 'InstanceFiling', Reason: 'The value lacks a dei:DocumentType', Input: 'xbrl'";

        let result = parse(&input, &registry)
            .expect_err("Given a document without a document type, the parse should always fail")
            .to_string();

        assert_eq!(result, expected_result);
    }
}
