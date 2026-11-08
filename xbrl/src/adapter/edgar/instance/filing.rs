//! # Instance Filing
//!
//! Maps the cover facts of an instance document, in the `dei` namespace, into one [`Filing`].

use roxmltree::Node;

use super::InstanceFiling;
use super::document::{Document, XML_TRUE, date, local_name, name as xml_name, text};
use crate::core::entity_name::EntityName;
use crate::core::filing::{CompanyAttributes, Filing, Ticker};
use crate::core::fiscal_period::FiscalPeriod;
use crate::core::fiscal_year::FiscalYear;
use crate::core::form::Form;
use crate::core::invalid_value::{InvalidValue, InvalidValueReason};

/// The start of each namespace URI of the SEC cover taxonomy, as in `http://xbrl.sec.gov/dei/2026`.
pub const DEI_NAMESPACE_START: &str = "http://xbrl.sec.gov/dei/";

/// The names of the cover facts this module reads. `DOCUMENT_TYPE` keeps its prefix, because a
/// finding names it.
mod name {
    pub const DOCUMENT_TYPE: &str = "dei:DocumentType";
    pub const AMENDMENT_FLAG: &str = "AmendmentFlag";
    pub const DOCUMENT_PERIOD_END_DATE: &str = "DocumentPeriodEndDate";
    pub const DOCUMENT_FISCAL_YEAR_FOCUS: &str = "DocumentFiscalYearFocus";
    pub const DOCUMENT_FISCAL_PERIOD_FOCUS: &str = "DocumentFiscalPeriodFocus";
    pub const ENTITY_REGISTRANT_NAME: &str = "EntityRegistrantName";
    pub const CURRENT_FISCAL_YEAR_END_DATE: &str = "CurrentFiscalYearEndDate";
    pub const TRADING_SYMBOL: &str = "TradingSymbol";
    pub const SECURITY_EXCHANGE_NAME: &str = "SecurityExchangeName";
}

/// Maps the cover facts into the filing.
///
/// The ID and the filed date come from the input, because the document holds neither. The entity
/// is the entity of the context of `dei:DocumentType`. A cover fact counts only in a context with no
/// dimensions, except `dei:TradingSymbol`, which names a class of stock. The document holds no
/// industry code and no acceptance time.
///
/// # Errors
///
/// Returns [`InvalidValue`] if the document has no `dei:DocumentType` in a context with no
/// dimensions, or if a cover fact breaks a rule of a core type.
pub fn filing(
    input: &InstanceFiling<'_>,
    document: &Document<'_, '_>,
) -> Result<Filing, InvalidValue> {
    let cover = Cover { document };
    let document_type = cover.fact(local_name(name::DOCUMENT_TYPE)).ok_or_else(|| {
        InvalidValue::new(
            InstanceFiling::TYPE_NAME,
            InvalidValueReason::MissingPart(name::DOCUMENT_TYPE),
            xml_name::ROOT,
        )
    })?;
    let form = Form::new(text(document_type).trim())?;
    let entity = document
        .context(
            document_type
                .attribute(xml_name::CONTEXT_REF)
                .unwrap_or_default(),
        )?
        .entity
        .clone();
    let is_amendment = form.is_amendment()
        || cover
            .text(name::AMENDMENT_FLAG)
            .is_some_and(|flag| XML_TRUE.contains(&flag.as_str()));
    Ok(Filing {
        id: input.id.clone(),
        entity,
        form,
        is_amendment,
        filed: input.filed,
        accepted: None,
        period_of_report: cover
            .text(name::DOCUMENT_PERIOD_END_DATE)
            .map(|text| date(&text))
            .transpose()?,
        fiscal_year: cover
            .text(name::DOCUMENT_FISCAL_YEAR_FOCUS)
            .map(|text| {
                text.parse().map(FiscalYear::new).map_err(|_| {
                    InvalidValue::new(
                        InstanceFiling::TYPE_NAME,
                        InvalidValueReason::NotANumber,
                        text,
                    )
                })
            })
            .transpose()?,
        fiscal_period: cover
            .text(name::DOCUMENT_FISCAL_PERIOD_FOCUS)
            .map(FiscalPeriod::new)
            .transpose()?,
        company: CompanyAttributes {
            name: cover
                .text(name::ENTITY_REGISTRANT_NAME)
                .map(EntityName::new),
            industry: None,
            tickers: cover.tickers(),
            fiscal_year_end: cover.text(name::CURRENT_FISCAL_YEAR_END_DATE),
        },
    })
}

/// The cover facts of one document.
struct Cover<'d, 'a, 'input> {
    document: &'d Document<'a, 'input>,
}

impl<'a, 'input> Cover<'_, 'a, 'input> {
    /// Returns the first cover fact with the local name in a context with no dimensions.
    fn fact(&self, local_name: &str) -> Option<Node<'a, 'input>> {
        self.all(local_name).find(|fact| {
            self.document
                .context(fact.attribute(xml_name::CONTEXT_REF).unwrap_or_default())
                .is_ok_and(|context| context.dimensions.is_empty())
        })
    }

    /// Returns the trimmed text of [`Cover::fact`].
    fn text(&self, local_name: &str) -> Option<String> {
        self.fact(local_name)
            .map(|fact| text(fact).trim().to_owned())
    }

    /// Returns every cover fact with the local name, in document order.
    fn all(&self, local_name: &str) -> impl Iterator<Item = Node<'a, 'input>> {
        self.document.facts().filter(move |fact| {
            fact.tag_name().name() == local_name
                && fact
                    .tag_name()
                    .namespace()
                    .is_some_and(|namespace| namespace.starts_with(DEI_NAMESPACE_START))
        })
    }

    /// Returns one ticker per `dei:TradingSymbol`, with the exchange of the same context.
    fn tickers(&self) -> Vec<Ticker> {
        self.all(name::TRADING_SYMBOL)
            .map(|symbol| {
                let context = symbol.attribute(xml_name::CONTEXT_REF);
                Ticker {
                    symbol: text(symbol).trim().to_owned(),
                    exchange: self
                        .all(name::SECURITY_EXCHANGE_NAME)
                        .find(|exchange| exchange.attribute(xml_name::CONTEXT_REF) == context)
                        .map(|exchange| text(exchange).trim().to_owned()),
                }
            })
            .collect()
    }
}
