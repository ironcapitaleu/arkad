//! # Element Schema
//!
//! Reads the concepts of an XBRL element schema, with their data type, balance, and period type.

use std::path::Path;

use roxmltree::{Document, Node};

use crate::error::GeneratorError;

/// The namespace of XML Schema.
const XML_SCHEMA_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema";

/// The namespace of the XBRL instance, which holds `balance` and `periodType`.
const XBRL_INSTANCE_NAMESPACE: &str = "http://www.xbrl.org/2003/instance";

/// The local name of a schema element declaration.
const ELEMENT_TAG: &str = "element";

/// The attribute that holds the local name of a concept.
const NAME_ATTRIBUTE: &str = "name";

/// The attribute that holds the data type of a concept.
const TYPE_ATTRIBUTE: &str = "type";

/// The attribute that makes a schema element a concept.
const SUBSTITUTION_GROUP_ATTRIBUTE: &str = "substitutionGroup";

/// The XBRL attribute that holds the balance of a monetary concept.
const BALANCE_ATTRIBUTE: &str = "balance";

/// The XBRL attribute that holds the period type of a concept.
const PERIOD_TYPE_ATTRIBUTE: &str = "periodType";

/// The balance of a monetary concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Balance {
    /// The value of the concept increases with a debit.
    Debit,
    /// The value of the concept increases with a credit.
    Credit,
}

impl Balance {
    /// The value of `balance` for [`Balance::Debit`].
    const DEBIT: &'static str = "debit";
    /// The value of `balance` for [`Balance::Credit`].
    const CREDIT: &'static str = "credit";

    /// Returns the [`Balance`] of a `balance` value, or `None` if XBRL defines no such value.
    fn from_attribute(value: &str) -> Option<Self> {
        match value {
            Self::DEBIT => Some(Self::Debit),
            Self::CREDIT => Some(Self::Credit),
            _ => None,
        }
    }
}

/// The period type of a concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PeriodType {
    /// The value holds at one date.
    Instant,
    /// The value holds over a period.
    Duration,
}

impl PeriodType {
    /// The value of `periodType` for [`PeriodType::Instant`].
    const INSTANT: &'static str = "instant";
    /// The value of `periodType` for [`PeriodType::Duration`].
    const DURATION: &'static str = "duration";

    /// Returns the [`PeriodType`] of a `periodType` value, or `None` if XBRL defines no such value.
    fn from_attribute(value: &str) -> Option<Self> {
        match value {
            Self::INSTANT => Some(Self::Instant),
            Self::DURATION => Some(Self::Duration),
            _ => None,
        }
    }
}

/// One concept of an element schema.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Concept {
    /// The local name, for example `Assets`.
    pub name: String,
    /// The data type as the schema writes it, for example `xbrli:monetaryItemType`.
    pub data_type: String,
    /// The balance, if the concept has one.
    pub balance: Option<Balance>,
    /// The period type.
    pub period_type: PeriodType,
}

/// Returns the concepts of the element schema `schema`, in the order of the file.
///
/// A concept is a top-level `xs:element` with a `substitutionGroup`. Other declarations, such as
/// the domain helpers of typed dimensions, are skipped.
///
/// # Errors
///
/// - [`GeneratorError::InvalidXml`] if `schema` is not well-formed XML.
/// - [`GeneratorError::MissingAttribute`] if a concept has no name, type, or period type.
/// - [`GeneratorError::InvalidAttributeValue`] if a balance or period type is unknown.
pub fn concepts(path: &Path, schema: &str) -> Result<Vec<Concept>, GeneratorError> {
    let document = Document::parse(schema).map_err(|error| GeneratorError::InvalidXml {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    document
        .root_element()
        .children()
        .filter(|node| node.has_tag_name((XML_SCHEMA_NAMESPACE, ELEMENT_TAG)))
        .filter(|node| node.attribute(SUBSTITUTION_GROUP_ATTRIBUTE).is_some())
        .map(|node| concept(path, node))
        .collect()
}

/// Reads one concept declaration.
fn concept(path: &Path, node: Node<'_, '_>) -> Result<Concept, GeneratorError> {
    // The position of a node is a byte offset. The line costs a scan of the file, so only an
    // error computes it.
    let line = || node.document().text_pos_at(node.range().start).row;
    let required = |name: &'static str, value: Option<&str>| {
        value
            .map(str::to_string)
            .ok_or_else(|| GeneratorError::MissingAttribute {
                path: path.to_path_buf(),
                line: line(),
                element: ELEMENT_TAG.to_string(),
                attribute: name,
            })
    };
    let invalid = |attribute: &'static str, value: &str| GeneratorError::InvalidAttributeValue {
        path: path.to_path_buf(),
        line: line(),
        attribute,
        value: value.to_string(),
    };
    let name = required(NAME_ATTRIBUTE, node.attribute(NAME_ATTRIBUTE))?;
    let data_type = required(TYPE_ATTRIBUTE, node.attribute(TYPE_ATTRIBUTE))?;
    let period_type_value = required(
        PERIOD_TYPE_ATTRIBUTE,
        node.attribute((XBRL_INSTANCE_NAMESPACE, PERIOD_TYPE_ATTRIBUTE)),
    )?;
    let period_type = PeriodType::from_attribute(&period_type_value)
        .ok_or_else(|| invalid(PERIOD_TYPE_ATTRIBUTE, &period_type_value))?;
    let balance = node
        .attribute((XBRL_INSTANCE_NAMESPACE, BALANCE_ATTRIBUTE))
        .map(|value| {
            Balance::from_attribute(value).ok_or_else(|| invalid(BALANCE_ATTRIBUTE, value))
        })
        .transpose()?;
    Ok(Concept {
        name,
        data_type,
        balance,
        period_type,
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const PATH: &str = "us-gaap-2026.xsd";

    fn schema(elements: &str) -> String {
        format!(
            "<xs:schema xmlns:xs='http://www.w3.org/2001/XMLSchema' \
             xmlns:xbrli='http://www.xbrl.org/2003/instance'>{elements}</xs:schema>"
        )
    }

    #[test]
    fn should_read_type_balance_and_period_type_for_monetary_concept() {
        let text = schema(
            "<xs:element id='us-gaap_Assets' name='Assets' substitutionGroup='xbrli:item' \
             type='xbrli:monetaryItemType' xbrli:balance='debit' xbrli:periodType='instant' />",
        );

        let expected_result = vec![Concept {
            name: "Assets".to_string(),
            data_type: "xbrli:monetaryItemType".to_string(),
            balance: Some(Balance::Debit),
            period_type: PeriodType::Instant,
        }];

        let result = concepts(Path::new(PATH), &text)
            .expect("Given a schema with one valid concept, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_declaration_when_it_has_no_substitution_group() {
        let text = schema(
            "<xs:element id='us-gaap_date.domain' name='date.domain' type='xs:date' />\
             <xs:element id='us-gaap_Revenues' name='Revenues' substitutionGroup='xbrli:item' \
             type='xbrli:monetaryItemType' xbrli:balance='credit' xbrli:periodType='duration' />",
        );

        let expected_result = vec![Concept {
            name: "Revenues".to_string(),
            data_type: "xbrli:monetaryItemType".to_string(),
            balance: Some(Balance::Credit),
            period_type: PeriodType::Duration,
        }];

        let result = concepts(Path::new(PATH), &text)
            .expect("Given a schema with one valid concept, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_attribute_when_concept_has_no_period_type() {
        let text = schema(
            "<xs:element name='Assets' substitutionGroup='xbrli:item' \
             type='xbrli:monetaryItemType' />",
        );

        let expected_result = Some(
            "[MissingAttribute] Required attribute 'periodType' not found on element 'element', \
             Line: '1', Path: 'us-gaap-2026.xsd'"
                .to_string(),
        );

        let result = concepts(Path::new(PATH), &text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_attribute_value_when_balance_is_unknown() {
        let text = schema(
            "<xs:element name='Assets' substitutionGroup='xbrli:item' \
             type='xbrli:monetaryItemType' xbrli:balance='both' xbrli:periodType='instant' />",
        );

        let expected_result = Some(
            "[InvalidAttributeValue] Attribute 'balance' holds an unknown value, Value: 'both', \
             Line: '1', Path: 'us-gaap-2026.xsd'"
                .to_string(),
        );

        let result = concepts(Path::new(PATH), &text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_no_balance_when_concept_has_no_balance_attribute() {
        let text = schema(
            "<xs:element name='AmendmentDescription' substitutionGroup='xbrli:item' \
             type='xbrli:stringItemType' xbrli:periodType='duration' />",
        );

        let expected_result = vec![Concept {
            name: "AmendmentDescription".to_string(),
            data_type: "xbrli:stringItemType".to_string(),
            balance: None,
            period_type: PeriodType::Duration,
        }];

        let result = concepts(Path::new(PATH), &text)
            .expect("Given a schema with one valid concept, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_skip_declaration_when_it_is_nested_in_a_type() {
        let text = schema(
            "<xs:complexType name='Wrapper'><xs:sequence>\
             <xs:element name='Inner' substitutionGroup='xbrli:item' type='xs:string' />\
             </xs:sequence></xs:complexType>",
        );

        let expected_result = Vec::<Concept>::new();

        let result = concepts(Path::new(PATH), &text)
            .expect("Given a schema with no top-level concept, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_attribute_value_when_period_type_is_unknown() {
        let text = schema(
            "<xs:element name='Assets' substitutionGroup='xbrli:item' \
             type='xbrli:monetaryItemType' xbrli:periodType='forever' />",
        );

        let expected_result = Some(
            "[InvalidAttributeValue] Attribute 'periodType' holds an unknown value, \
             Value: 'forever', Line: '1', Path: 'us-gaap-2026.xsd'"
                .to_string(),
        );

        let result = concepts(Path::new(PATH), &text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_xml_when_prefix_has_no_namespace() {
        let text = "<xs:schema />";

        let expected_result = Some(
            "[InvalidXml] Failed to parse XML, Reason: 'an unknown namespace prefix 'xs' at 1:2', \
             Path: 'us-gaap-2026.xsd'"
                .to_string(),
        );

        let result = concepts(Path::new(PATH), text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }
}
