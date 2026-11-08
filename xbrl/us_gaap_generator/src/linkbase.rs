//! # Linkbases
//!
//! Reads the standard labels of a label linkbase and the arcs of a calculation linkbase.
//!
//! Both linkbases hold extended links. Each link has locators that point to concepts, and arcs
//! that join the labels of its locators and resources. The generator resolves every arc through
//! these labels and never guesses a concept from the text of a label.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;

use roxmltree::{Document, Node};

use crate::error::GeneratorError;

/// The namespace of the XBRL linkbase elements.
const LINK_NAMESPACE: &str = "http://www.xbrl.org/2003/linkbase";

/// The namespace of the `XLink` attributes.
const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";

/// The local name of a locator.
const LOCATOR_TAG: &str = "loc";

/// The local name of a label resource.
const LABEL_TAG: &str = "label";

/// The local name of an arc of a label linkbase.
const LABEL_ARC_TAG: &str = "labelArc";

/// The local name of an arc of a calculation linkbase.
const CALCULATION_ARC_TAG: &str = "calculationArc";

/// The `XLink` attribute that names a locator or a resource.
const LABEL_ATTRIBUTE: &str = "label";

/// The `XLink` attribute that points a locator to a concept.
const HREF_ATTRIBUTE: &str = "href";

/// The `XLink` attribute that holds the role of an extended link or of a label resource.
const ROLE_ATTRIBUTE: &str = "role";

/// The `XLink` attribute that holds the kind of an `XLink` element.
const TYPE_ATTRIBUTE: &str = "type";

/// The `xlink:type` of an extended link.
const EXTENDED_TYPE: &str = "extended";

/// The `XLink` attribute that names the start of an arc.
const FROM_ATTRIBUTE: &str = "from";

/// The `XLink` attribute that names the end of an arc.
const TO_ATTRIBUTE: &str = "to";

/// The attribute that holds the weight of a calculation arc.
const WEIGHT_ATTRIBUTE: &str = "weight";

/// The role of the standard label.
const STANDARD_LABEL_ROLE: &str = "http://www.xbrl.org/2003/role/label";

/// The character before the fragment of a locator `href`.
const FRAGMENT_SEPARATOR: char = '#';

/// The character between the family and the name in the fragment, as in `us-gaap_Assets`.
const ID_SEPARATOR: char = '_';

/// The character between the family and the name in the text of a concept.
const CONCEPT_SEPARATOR: char = ':';

/// A concept that a locator points to: its namespace family and its local name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConceptRef {
    /// The namespace family, for example `us-gaap`.
    pub family: String,
    /// The local name, for example `Assets`.
    pub name: String,
}

impl fmt::Display for ConceptRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{CONCEPT_SEPARATOR}{}", self.family, self.name)
    }
}

/// One summation arc of a calculation linkbase.
///
/// A concept can be the total of different children in different networks. For example,
/// `Revenues` sums other children in the income statement than in the statement of comprehensive
/// income. So each arc keeps the role of its network, and a sum uses the arcs of one role.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalculationArc {
    /// The `xlink:role` of the calculation link that holds the arc: the network of the arc.
    pub role: String,
    /// The total.
    pub parent: ConceptRef,
    /// The part of the total.
    pub child: ConceptRef,
    /// The weight as the linkbase writes it, for example `-1.0`.
    pub weight: String,
}

/// Returns the standard label of each concept of the label linkbase `linkbase`.
///
/// # Errors
///
/// - [`GeneratorError::InvalidXml`] if `linkbase` is not well-formed XML.
/// - [`GeneratorError::MissingAttribute`] if a locator, resource, or arc lacks an `XLink`
///   attribute.
/// - [`GeneratorError::InvalidConceptReference`] if a locator does not point to a concept.
/// - [`GeneratorError::AmbiguousLocatorLabel`] if two locators share a label and differ in concept.
/// - [`GeneratorError::UnresolvedArcLabel`] if an arc names a label that its link does not carry.
/// - [`GeneratorError::RepeatedLabel`] if a concept has two standard labels.
pub fn standard_labels(
    path: &Path,
    linkbase: &str,
) -> Result<HashMap<ConceptRef, String>, GeneratorError> {
    let document = parse(path, linkbase)?;
    let mut labels = HashMap::new();
    for link in extended_links(&document) {
        let locators = locators(path, link)?;
        let mut resource_labels = HashSet::new();
        let mut standard_texts: HashMap<&str, Vec<&str>> = HashMap::new();
        for resource in children(link, LABEL_TAG) {
            let label = xlink_attribute(path, resource, LABEL_ATTRIBUTE)?;
            resource_labels.insert(label);
            if resource.attribute((XLINK_NAMESPACE, ROLE_ATTRIBUTE)) == Some(STANDARD_LABEL_ROLE) {
                let text = resource.text().unwrap_or_default();
                standard_texts.entry(label).or_default().push(text);
            }
        }
        for arc in children(link, LABEL_ARC_TAG) {
            let concept = resolve(path, &locators, xlink_attribute(path, arc, FROM_ATTRIBUTE)?)?;
            let to = xlink_attribute(path, arc, TO_ATTRIBUTE)?;
            if !resource_labels.contains(to) {
                return Err(unresolved(path, to));
            }
            for text in standard_texts.get(to).into_iter().flatten() {
                if labels
                    .insert(concept.clone(), (*text).to_string())
                    .is_some()
                {
                    return Err(GeneratorError::RepeatedLabel {
                        path: path.to_path_buf(),
                        concept: concept.to_string(),
                    });
                }
            }
        }
    }
    Ok(labels)
}

/// Returns the arcs of the calculation linkbase `linkbase`, in the order of the file.
///
/// # Errors
///
/// - [`GeneratorError::InvalidXml`] if `linkbase` is not well-formed XML.
/// - [`GeneratorError::MissingAttribute`] if a link, locator, or arc lacks an attribute.
/// - [`GeneratorError::InvalidConceptReference`] if a locator does not point to a concept.
/// - [`GeneratorError::AmbiguousLocatorLabel`] if two locators share a label and differ in concept.
/// - [`GeneratorError::UnresolvedArcLabel`] if an arc names a label that its link does not carry.
pub fn calculation_arcs(
    path: &Path,
    linkbase: &str,
) -> Result<Vec<CalculationArc>, GeneratorError> {
    let document = parse(path, linkbase)?;
    let mut arcs = Vec::new();
    for link in extended_links(&document) {
        let role = xlink_attribute(path, link, ROLE_ATTRIBUTE)?;
        let locators = locators(path, link)?;
        for arc in children(link, CALCULATION_ARC_TAG) {
            let weight = arc
                .attribute(WEIGHT_ATTRIBUTE)
                .ok_or_else(|| missing(path, arc, WEIGHT_ATTRIBUTE))?;
            arcs.push(CalculationArc {
                role: role.to_string(),
                parent: resolve(path, &locators, xlink_attribute(path, arc, FROM_ATTRIBUTE)?)?,
                child: resolve(path, &locators, xlink_attribute(path, arc, TO_ATTRIBUTE)?)?,
                weight: weight.to_string(),
            });
        }
    }
    Ok(arcs)
}

/// Parses `text` as XML.
fn parse<'input>(path: &Path, text: &'input str) -> Result<Document<'input>, GeneratorError> {
    Document::parse(text).map_err(|error| GeneratorError::InvalidXml {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })
}

/// Returns the extended links of a linkbase: the children of its root with `xlink:type`
/// `extended`. The role references beside them are skipped.
fn extended_links<'a, 'input>(
    document: &'a Document<'input>,
) -> impl Iterator<Item = Node<'a, 'input>> {
    document
        .root_element()
        .children()
        .filter(|node| node.attribute((XLINK_NAMESPACE, TYPE_ATTRIBUTE)) == Some(EXTENDED_TYPE))
}

/// Returns the children of `link` with the linkbase tag `tag`.
fn children<'a, 'input>(
    link: Node<'a, 'input>,
    tag: &'static str,
) -> impl Iterator<Item = Node<'a, 'input>> {
    link.children()
        .filter(move |node| node.has_tag_name((LINK_NAMESPACE, tag)))
}

/// Returns the concept of each locator of `link`, by the label of the locator.
///
/// `XLink` lets two locators share a label. The generator accepts this only if both point to the
/// same concept, so an arc never stands for two concepts.
fn locators<'a>(
    path: &Path,
    link: Node<'a, '_>,
) -> Result<HashMap<&'a str, ConceptRef>, GeneratorError> {
    let mut locators = HashMap::new();
    for locator in children(link, LOCATOR_TAG) {
        let label = xlink_attribute(path, locator, LABEL_ATTRIBUTE)?;
        let concept = concept_of(path, xlink_attribute(path, locator, HREF_ATTRIBUTE)?)?;
        if locators
            .insert(label, concept.clone())
            .is_some_and(|other| other != concept)
        {
            return Err(GeneratorError::AmbiguousLocatorLabel {
                path: path.to_path_buf(),
                line: line_of(locator),
                label: label.to_string(),
            });
        }
    }
    Ok(locators)
}

/// Returns the concept of a locator `href`, such as `../elts/us-gaap-2026.xsd#us-gaap_Assets`.
fn concept_of(path: &Path, href: &str) -> Result<ConceptRef, GeneratorError> {
    href.rsplit_once(FRAGMENT_SEPARATOR)
        .and_then(|(_, fragment)| fragment.split_once(ID_SEPARATOR))
        .filter(|(family, name)| !family.is_empty() && !name.is_empty())
        .map(|(family, name)| ConceptRef {
            family: family.to_string(),
            name: name.to_string(),
        })
        .ok_or_else(|| GeneratorError::InvalidConceptReference {
            path: path.to_path_buf(),
            href: href.to_string(),
        })
}

/// Returns the concept of the locator with the label `label`.
fn resolve(
    path: &Path,
    locators: &HashMap<&str, ConceptRef>,
    label: &str,
) -> Result<ConceptRef, GeneratorError> {
    locators
        .get(label)
        .cloned()
        .ok_or_else(|| unresolved(path, label))
}

/// Returns the error for an arc that names the label `label`, which its link does not carry.
fn unresolved(path: &Path, label: &str) -> GeneratorError {
    GeneratorError::UnresolvedArcLabel {
        path: path.to_path_buf(),
        label: label.to_string(),
    }
}

/// Returns the line of `node` in its file.
fn line_of(node: Node<'_, '_>) -> u32 {
    node.document().text_pos_at(node.range().start).row
}

/// Returns the `XLink` attribute `name` of `node`.
fn xlink_attribute<'a>(
    path: &Path,
    node: Node<'a, '_>,
    name: &'static str,
) -> Result<&'a str, GeneratorError> {
    node.attribute((XLINK_NAMESPACE, name))
        .ok_or_else(|| missing(path, node, name))
}

/// Returns the error for the attribute `name` that `node` lacks.
fn missing(path: &Path, node: Node<'_, '_>, name: &'static str) -> GeneratorError {
    GeneratorError::MissingAttribute {
        path: path.to_path_buf(),
        line: line_of(node),
        element: node.tag_name().name().to_string(),
        attribute: name,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const PATH: &str = "linkbase.xml";

    fn linkbase(link: &str) -> String {
        format!(
            "<link:linkbase xmlns:link='http://www.xbrl.org/2003/linkbase' \
             xmlns:xlink='http://www.w3.org/1999/xlink'>{link}</link:linkbase>"
        )
    }

    fn concept(family: &str, name: &str) -> ConceptRef {
        ConceptRef {
            family: family.to_string(),
            name: name.to_string(),
        }
    }

    #[test]
    fn should_return_standard_label_when_concept_also_has_total_label() {
        let text = linkbase(
            "<link:labelLink xlink:type='extended'>\
             <link:loc xlink:href='us-gaap-2026.xsd#us-gaap_Assets' xlink:label='loc_A' />\
             <link:label xlink:label='lab_A' xlink:role='http://www.xbrl.org/2003/role/label'>Assets</link:label>\
             <link:label xlink:label='lab_A' xlink:role='http://www.xbrl.org/2003/role/totalLabel'>Assets, Total</link:label>\
             <link:labelArc xlink:from='loc_A' xlink:to='lab_A' />\
             </link:labelLink>",
        );

        let expected_result = HashMap::from([(concept("us-gaap", "Assets"), "Assets".to_string())]);

        let result = standard_labels(Path::new(PATH), &text)
            .expect("Given a label link with one standard label, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_arcs_of_every_link_when_linkbase_has_two_links() {
        let text = linkbase(
            "<link:calculationLink xlink:type='extended' xlink:role='http://fasb.org/us-gaap/role/statement/A'>\
             <link:loc xlink:href='../elts/us-gaap-2026.xsd#us-gaap_Assets' xlink:label='loc_A' />\
             <link:loc xlink:href='../elts/us-gaap-2026.xsd#us-gaap_AssetsCurrent' xlink:label='loc_B' />\
             <link:calculationArc weight='1.0' xlink:from='loc_A' xlink:to='loc_B' />\
             </link:calculationLink>\
             <link:calculationLink xlink:type='extended' xlink:role='http://fasb.org/us-gaap/role/statement/B'>\
             <link:loc xlink:href='../elts/us-gaap-2026.xsd#us-gaap_Revenues' xlink:label='loc_A' />\
             <link:loc xlink:href='https://xbrl.fasb.org/srt/2026/elts/srt-2026.xsd#srt_CostOfGoodsAndServicesSold' xlink:label='loc_B' />\
             <link:calculationArc weight='-1.0' xlink:from='loc_A' xlink:to='loc_B' />\
             </link:calculationLink>",
        );

        let expected_result = vec![
            CalculationArc {
                role: "http://fasb.org/us-gaap/role/statement/A".to_string(),
                parent: concept("us-gaap", "Assets"),
                child: concept("us-gaap", "AssetsCurrent"),
                weight: "1.0".to_string(),
            },
            CalculationArc {
                role: "http://fasb.org/us-gaap/role/statement/B".to_string(),
                parent: concept("us-gaap", "Revenues"),
                child: concept("srt", "CostOfGoodsAndServicesSold"),
                weight: "-1.0".to_string(),
            },
        ];

        let result = calculation_arcs(Path::new(PATH), &text)
            .expect("Given two valid calculation links, the read should always succeed");

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unresolved_arc_label_when_arc_names_no_locator() {
        let text = linkbase(
            "<link:calculationLink xlink:type='extended' xlink:role='http://fasb.org/us-gaap/role/statement/A'>\
             <link:loc xlink:href='../elts/us-gaap-2026.xsd#us-gaap_Assets' xlink:label='loc_A' />\
             <link:calculationArc weight='1.0' xlink:from='loc_A' xlink:to='loc_Missing' />\
             </link:calculationLink>",
        );

        let expected_result = Some(
            "[UnresolvedArcLabel] No locator or resource carries the label of an arc, \
             Label: 'loc_Missing', Path: 'linkbase.xml'"
                .to_string(),
        );

        let result = calculation_arcs(Path::new(PATH), &text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_invalid_concept_reference_when_href_has_no_fragment() {
        let text = linkbase(
            "<link:calculationLink xlink:type='extended' xlink:role='http://fasb.org/us-gaap/role/statement/A'>\
             <link:loc xlink:href='../elts/us-gaap-2026.xsd' xlink:label='loc_A' />\
             </link:calculationLink>",
        );

        let expected_result = Some(
            "[InvalidConceptReference] Locator does not point to a concept, \
             Href: '../elts/us-gaap-2026.xsd', Path: 'linkbase.xml'"
                .to_string(),
        );

        let result = calculation_arcs(Path::new(PATH), &text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_repeated_label_when_concept_has_two_standard_labels() {
        let text = linkbase(
            "<link:labelLink xlink:type='extended'>\
             <link:loc xlink:href='us-gaap-2026.xsd#us-gaap_Assets' xlink:label='loc_A' />\
             <link:label xlink:label='lab_A' xlink:role='http://www.xbrl.org/2003/role/label'>Assets</link:label>\
             <link:label xlink:label='lab_B' xlink:role='http://www.xbrl.org/2003/role/label'>Assets, Again</link:label>\
             <link:labelArc xlink:from='loc_A' xlink:to='lab_A' />\
             <link:labelArc xlink:from='loc_A' xlink:to='lab_B' />\
             </link:labelLink>",
        );

        let expected_result = Some(
            "[RepeatedLabel] Concept 'us-gaap:Assets' has more than one standard label, \
             Path: 'linkbase.xml'"
                .to_string(),
        );

        let result = standard_labels(Path::new(PATH), &text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_unresolved_arc_label_when_label_arc_names_no_resource() {
        let text = linkbase(
            "<link:labelLink xlink:type='extended'>\
             <link:loc xlink:href='us-gaap-2026.xsd#us-gaap_Assets' xlink:label='loc_A' />\
             <link:label xlink:label='lab_A' xlink:role='http://www.xbrl.org/2003/role/label'>Assets</link:label>\
             <link:labelArc xlink:from='loc_A' xlink:to='lab_Typo' />\
             </link:labelLink>",
        );

        let expected_result = Some(
            "[UnresolvedArcLabel] No locator or resource carries the label of an arc, \
             Label: 'lab_Typo', Path: 'linkbase.xml'"
                .to_string(),
        );

        let result = standard_labels(Path::new(PATH), &text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_missing_attribute_when_arc_has_no_weight() {
        let text = linkbase(
            "\n<link:calculationLink xlink:type='extended' xlink:role='http://fasb.org/us-gaap/role/statement/A'>\n\
             <link:loc xlink:href='../elts/us-gaap-2026.xsd#us-gaap_Assets' xlink:label='loc_A' />\n\
             <link:calculationArc xlink:from='loc_A' xlink:to='loc_A' />\n\
             </link:calculationLink>",
        );

        let expected_result = Some(
            "[MissingAttribute] Required attribute 'weight' not found on element 'calculationArc', \
             Line: '4', Path: 'linkbase.xml'"
                .to_string(),
        );

        let result = calculation_arcs(Path::new(PATH), &text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_ambiguous_locator_label_when_two_locators_share_a_label() {
        let text = linkbase(
            "<link:calculationLink xlink:type='extended' xlink:role='http://fasb.org/us-gaap/role/statement/A'>\
             <link:loc xlink:href='../elts/us-gaap-2026.xsd#us-gaap_Assets' xlink:label='loc_A' />\
             <link:loc xlink:href='../elts/us-gaap-2026.xsd#us-gaap_Liabilities' xlink:label='loc_A' />\
             </link:calculationLink>",
        );

        let expected_result = Some(
            "[AmbiguousLocatorLabel] Two locators share a label and differ in concept, \
             Label: 'loc_A', Line: '1', Path: 'linkbase.xml'"
                .to_string(),
        );

        let result = calculation_arcs(Path::new(PATH), &text)
            .err()
            .map(|error| error.to_string());

        assert_eq!(result, expected_result);
    }
}
