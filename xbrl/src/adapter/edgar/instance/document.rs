//! # Instance Document
//!
//! Reads the contexts, the units, the footnotes, and the extension namespace of one XBRL instance
//! document, and lists its facts.
//!
//! The parser maps each fact through the context and the unit it names. A context or a unit that
//! breaks a rule of a core type keeps its error, so only the facts that name it give a finding.

use std::collections::HashMap;

use chrono::NaiveDate;
use roxmltree::Node;

use crate::adapter::edgar::company_facts::CIK_SCHEME;
use crate::adapter::edgar::notes::row::without_prefix;
use crate::core::fact::{Context, Dimensions};
use crate::core::filing::EntityId;
use crate::core::invalid_value::{InvalidValue, InvalidValueReason};
use crate::core::period::{Duration, Period};
use crate::core::unit::{Measure, Unit};

/// The namespace of the XBRL instance elements, such as `context` and `unit`.
pub const XBRLI_NAMESPACE: &str = "http://www.xbrl.org/2003/instance";

/// The namespace of the XBRL link elements, such as `schemaRef` and `footnoteLink`.
pub const LINK_NAMESPACE: &str = "http://www.xbrl.org/2003/linkbase";

/// The namespace of the `XLink` attributes, such as `href` and `label`.
pub const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";

/// The namespace of the XBRL dimension members.
pub const XBRLDI_NAMESPACE: &str = "http://xbrl.org/2006/xbrldi";

/// The arc role of a link from a fact to its footnote.
pub const FACT_FOOTNOTE_ARCROLE: &str = "http://www.xbrl.org/2003/arcrole/fact-footnote";

/// The format of an XBRL date, for example `2026-09-30`.
pub const DATE_FORMAT: &str = "%Y-%m-%d";

/// The prefix of an explicit axis that the SEC drops, as in `StatementBusinessSegmentsAxis`.
pub const STATEMENT_PREFIX: &str = "Statement";

/// The suffix of an axis that the SEC drops.
pub const AXIS_SUFFIX: &str = "Axis";

/// The suffixes of an explicit member that the SEC drops. The first that matches is dropped.
pub const MEMBER_SUFFIXES: [&str; 2] = ["Member", "Domain"];

/// The element and attribute names this module reads.
mod name {
    pub const CONTEXT: &str = "context";
    pub const ENTITY: &str = "entity";
    pub const IDENTIFIER: &str = "identifier";
    pub const SCHEME: &str = "scheme";
    pub const SEGMENT: &str = "segment";
    pub const SCENARIO: &str = "scenario";
    pub const PERIOD: &str = "period";
    pub const INSTANT: &str = "instant";
    pub const START_DATE: &str = "startDate";
    pub const END_DATE: &str = "endDate";
    pub const UNIT: &str = "unit";
    pub const MEASURE: &str = "measure";
    pub const DIVIDE: &str = "divide";
    pub const UNIT_NUMERATOR: &str = "unitNumerator";
    pub const UNIT_DENOMINATOR: &str = "unitDenominator";
    pub const EXPLICIT_MEMBER: &str = "explicitMember";
    pub const TYPED_MEMBER: &str = "typedMember";
    pub const DIMENSION: &str = "dimension";
    pub const SCHEMA_REF: &str = "schemaRef";
    pub const FOOTNOTE_LINK: &str = "footnoteLink";
    pub const LOC: &str = "loc";
    pub const FOOTNOTE: &str = "footnote";
    pub const FOOTNOTE_ARC: &str = "footnoteArc";
    pub const HREF: &str = "href";
    pub const LABEL: &str = "label";
    pub const FROM: &str = "from";
    pub const TO: &str = "to";
    pub const ARCROLE: &str = "arcrole";
    pub const ID: &str = "id";
    pub const CONTEXT_REF: &str = "contextRef";
}

/// The character between a prefix and a local name, as in `us-gaap:Revenues`.
pub const PREFIX_SEPARATOR: char = ':';

/// The character before the fact ID in the `href` of a footnote locator, as in `#f-12`.
const FRAGMENT_SEPARATOR: char = '#';

/// The character between the folders of a path, as in the `href` of a `schemaRef`.
const PATH_SEPARATOR: char = '/';

/// The character after the prefix in the name of a schema file, as in `fdxf-20260930.xsd`.
const SCHEMA_NAME_SEPARATOR: char = '-';

/// The text between the words of a footnote.
const WORD_SEPARATOR: &str = " ";

/// The contexts, the units, the footnotes, and the extension namespace of one instance document.
pub struct Document<'a, 'input> {
    root: Node<'a, 'input>,
    contexts: HashMap<&'a str, Result<Context, InvalidValue>>,
    units: HashMap<&'a str, Result<Unit, InvalidValue>>,
    footnotes: HashMap<&'a str, Vec<String>>,
    extension: Option<&'a str>,
}

impl<'a, 'input> Document<'a, 'input> {
    /// The type name that [`InvalidValue`] names when a context or a unit breaks a rule.
    const TYPE_NAME: &'static str = "InstanceDocument";

    /// Reads the contexts, the units, the footnotes, and the extension namespace below the root
    /// element.
    #[must_use]
    pub fn read(root: Node<'a, 'input>) -> Self {
        let mut contexts = HashMap::new();
        let mut units = HashMap::new();
        let mut footnotes = HashMap::new();
        let mut extension = None;
        for node in root.children().filter(Node::is_element) {
            let id = node.attribute(name::ID).unwrap_or_default();
            if node.has_tag_name((XBRLI_NAMESPACE, name::CONTEXT)) {
                contexts.insert(id, context(node));
            } else if node.has_tag_name((XBRLI_NAMESPACE, name::UNIT)) {
                units.insert(id, unit(node));
            } else if node.has_tag_name((LINK_NAMESPACE, name::FOOTNOTE_LINK)) {
                read_footnote_link(node, &mut footnotes);
            } else if node.has_tag_name((LINK_NAMESPACE, name::SCHEMA_REF)) {
                extension = extension.or_else(|| extension_namespace(root, node));
            }
        }
        Self {
            root,
            contexts,
            units,
            footnotes,
            extension,
        }
    }

    /// Returns the facts in document order: the elements below the root that name a context.
    pub fn facts(&self) -> impl Iterator<Item = Node<'a, 'input>> {
        self.root
            .children()
            .filter(|node| node.attribute(name::CONTEXT_REF).is_some())
    }

    /// Returns the context with the ID.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if no context has the ID, or if the context breaks a rule of a core
    /// type.
    pub fn context(&self, id: &str) -> Result<&Context, InvalidValue> {
        self.contexts.get(id).map_or_else(
            || Err(missing_element(name::CONTEXT, id)),
            |context| context.as_ref().map_err(Clone::clone),
        )
    }

    /// Returns the unit with the ID.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidValue`] if no unit has the ID, or if the unit breaks a rule of a core type.
    pub fn unit(&self, id: &str) -> Result<&Unit, InvalidValue> {
        self.units.get(id).map_or_else(
            || Err(missing_element(name::UNIT, id)),
            |unit| unit.as_ref().map_err(Clone::clone),
        )
    }

    /// Returns the footnotes of the fact with the ID, in the order of the links.
    #[must_use]
    pub fn footnotes(&self, fact_id: &str) -> Vec<String> {
        self.footnotes.get(fact_id).cloned().unwrap_or_default()
    }

    /// Returns `true` if the schema of the filing defines the namespace.
    ///
    /// The instance names its schema file, not the namespace the schema defines. The SEC names the
    /// schema file `<prefix>-<date>.xsd`, so the namespace whose prefix names the file is the
    /// extension namespace.
    #[must_use]
    pub fn is_extension(&self, namespace: &str) -> bool {
        self.extension == Some(namespace)
    }
}

/// Returns the local name of a qualified name, as in `Revenues` for `us-gaap:Revenues`.
#[must_use]
pub fn local_name(qualified_name: &str) -> &str {
    qualified_name
        .split_once(PREFIX_SEPARATOR)
        .map_or(qualified_name, |(_, local_name)| local_name)
}

/// Returns the text of the node and its descendants, joined.
#[must_use]
pub fn text(node: Node<'_, '_>) -> String {
    node.descendants()
        .filter(Node::is_text)
        .filter_map(|node| node.text())
        .collect()
}

fn context(node: Node<'_, '_>) -> Result<Context, InvalidValue> {
    let entity = required_child(node, name::ENTITY)?;
    let identifier = required_child(entity, name::IDENTIFIER)?;
    let pairs = [child(entity, name::SEGMENT), child(node, name::SCENARIO)]
        .into_iter()
        .flatten()
        .flat_map(|container| container.children().filter(Node::is_element))
        .map(dimension)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Context {
        entity: entity_id(
            identifier.attribute(name::SCHEME).unwrap_or_default(),
            text(identifier).trim(),
        )?,
        period: period(required_child(node, name::PERIOD)?)?,
        dimensions: Dimensions::new(pairs)?,
    })
}

/// Returns the entity. A CIK gets ten digits, as the Notes parser writes it.
fn entity_id(scheme: &str, id: &str) -> Result<EntityId, InvalidValue> {
    if scheme != CIK_SCHEME {
        return EntityId::new(scheme, id);
    }
    let cik: u64 = id
        .parse()
        .map_err(|_| InvalidValue::new(Document::TYPE_NAME, InvalidValueReason::NotANumber, id))?;
    EntityId::new(CIK_SCHEME, format!("{cik:010}"))
}

fn period(node: Node<'_, '_>) -> Result<Period, InvalidValue> {
    if let Some(instant) = child(node, name::INSTANT) {
        return Ok(Period::Instant {
            end: date(&text(instant))?,
        });
    }
    let start = date(&text(required_child(node, name::START_DATE)?))?;
    let end = date(&text(required_child(node, name::END_DATE)?))?;
    Duration::from_dates(start, end).map(Period::Duration)
}

/// Returns the axis and the member, with the names shortened the way the SEC writes `dim.segments`.
///
/// An explicit axis loses a leading `Statement` and a trailing `Axis`, and its member loses a
/// trailing `Member` or `Domain`. A typed axis loses only a trailing `Axis`. A typed member keeps
/// its content, without the prefix of a qualified name, as the Notes parser reads it.
fn dimension(node: Node<'_, '_>) -> Result<(String, String), InvalidValue> {
    let axis = local_name(
        node.attribute(name::DIMENSION)
            .ok_or_else(|| missing_part(name::DIMENSION, node))?,
    );
    let axis_name = axis.strip_suffix(AXIS_SUFFIX).unwrap_or(axis);
    if node.has_tag_name((XBRLDI_NAMESPACE, name::TYPED_MEMBER)) {
        let content = node.first_element_child().map(text).unwrap_or_default();
        return Ok((
            axis_name.to_owned(),
            without_prefix(content.trim()).to_owned(),
        ));
    }
    if !node.has_tag_name((XBRLDI_NAMESPACE, name::EXPLICIT_MEMBER)) {
        return Err(missing_part(name::EXPLICIT_MEMBER, node));
    }
    let member_text = text(node);
    let member = local_name(member_text.trim());
    let member_name = MEMBER_SUFFIXES
        .iter()
        .find_map(|suffix| member.strip_suffix(suffix))
        .unwrap_or(member);
    Ok((
        axis_name
            .strip_prefix(STATEMENT_PREFIX)
            .unwrap_or(axis_name)
            .to_owned(),
        member_name.to_owned(),
    ))
}

fn unit(node: Node<'_, '_>) -> Result<Unit, InvalidValue> {
    match child(node, name::DIVIDE) {
        Some(divide) => Unit::new(
            measures(required_child(divide, name::UNIT_NUMERATOR)?)?,
            measures(required_child(divide, name::UNIT_DENOMINATOR)?)?,
        ),
        None => Unit::new(measures(node)?, Vec::new()),
    }
}

/// Returns the measures below the node, without prefixes, as the Notes data set writes them.
fn measures(node: Node<'_, '_>) -> Result<Vec<Measure>, InvalidValue> {
    node.children()
        .filter(|child| child.has_tag_name((XBRLI_NAMESPACE, name::MEASURE)))
        .map(|measure| Measure::new(local_name(text(measure).trim())))
        .collect()
}

/// Adds the footnotes of one footnote link to the map from fact ID to footnotes.
///
/// The labels of a link name its locators and footnotes within the link only.
fn read_footnote_link<'a>(link: Node<'a, '_>, footnotes: &mut HashMap<&'a str, Vec<String>>) {
    let label = |node: &Node<'a, '_>| node.attribute((XLINK_NAMESPACE, name::LABEL));
    let mut facts: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut texts: HashMap<&str, String> = HashMap::new();
    for node in link.children().filter(Node::is_element) {
        if node.has_tag_name((LINK_NAMESPACE, name::LOC)) {
            if let (Some(label), Some(fact_id)) = (
                label(&node),
                node.attribute((XLINK_NAMESPACE, name::HREF))
                    .and_then(|href| href.split_once(FRAGMENT_SEPARATOR))
                    .map(|(_, fact_id)| fact_id),
            ) {
                facts.entry(label).or_default().push(fact_id);
            }
        } else if node.has_tag_name((LINK_NAMESPACE, name::FOOTNOTE))
            && let Some(label) = label(&node)
        {
            let words: Vec<String> = text(node).split_whitespace().map(str::to_owned).collect();
            texts.insert(label, words.join(WORD_SEPARATOR));
        }
    }
    let arcs = link.children().filter(|node| {
        node.has_tag_name((LINK_NAMESPACE, name::FOOTNOTE_ARC))
            && node.attribute((XLINK_NAMESPACE, name::ARCROLE)) == Some(FACT_FOOTNOTE_ARCROLE)
    });
    for arc in arcs {
        let from = arc.attribute((XLINK_NAMESPACE, name::FROM));
        let to = arc.attribute((XLINK_NAMESPACE, name::TO));
        if let (Some(fact_ids), Some(text)) = (
            from.and_then(|from| facts.get(from)),
            to.and_then(|to| texts.get(to)),
        ) {
            for fact_id in fact_ids {
                footnotes.entry(fact_id).or_default().push(text.clone());
            }
        }
    }
}

/// Returns the namespace whose prefix names the schema file, as `fdxf` names `fdxf-20260930.xsd`.
fn extension_namespace<'a>(root: Node<'a, '_>, schema_ref: Node<'a, '_>) -> Option<&'a str> {
    let href = schema_ref.attribute((XLINK_NAMESPACE, name::HREF))?;
    let file = href.rsplit(PATH_SEPARATOR).next()?;
    let (prefix, _) = file.split_once(SCHEMA_NAME_SEPARATOR)?;
    root.lookup_namespace_uri(Some(prefix))
}

fn child<'a, 'input>(node: Node<'a, 'input>, local_name: &str) -> Option<Node<'a, 'input>> {
    node.children()
        .find(|child| child.has_tag_name((XBRLI_NAMESPACE, local_name)))
}

fn required_child<'a, 'input>(
    node: Node<'a, 'input>,
    local_name: &'static str,
) -> Result<Node<'a, 'input>, InvalidValue> {
    child(node, local_name).ok_or_else(|| missing_part(local_name, node))
}

/// Parses an XBRL date, for example `2026-09-30`, with the whitespace around it removed.
///
/// # Errors
///
/// Returns [`InvalidValue`] if the text is not a date in [`DATE_FORMAT`].
pub fn date(text: &str) -> Result<NaiveDate, InvalidValue> {
    let text = text.trim();
    NaiveDate::parse_from_str(text, DATE_FORMAT).map_err(|_| {
        InvalidValue::new(
            Document::TYPE_NAME,
            InvalidValueReason::NotADate(DATE_FORMAT),
            text,
        )
    })
}

/// Returns the error for a node that lacks a child or an attribute. The input names the ID of the
/// nearest node that has one, such as the context.
fn missing_part(part: &'static str, node: Node<'_, '_>) -> InvalidValue {
    let id = node
        .ancestors()
        .find_map(|ancestor| ancestor.attribute(name::ID))
        .unwrap_or_default();
    InvalidValue::new(
        Document::TYPE_NAME,
        InvalidValueReason::MissingPart(part),
        id,
    )
}

fn missing_element(element: &'static str, id: &str) -> InvalidValue {
    InvalidValue::new(
        Document::TYPE_NAME,
        InvalidValueReason::NoMatchingElement(element),
        id,
    )
}
