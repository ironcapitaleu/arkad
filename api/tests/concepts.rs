use std::sync::Arc;

use api::AppState;
use api::config::InternalDocs;
use api::tests::fixtures::fake_token_verifier::FakeTokenVerifier;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn get_concepts() -> Response {
    let request = Request::get("/v1/concepts")
        .body(Body::empty())
        .expect("Given a hardcoded method and path, building the request should always succeed");
    api::router(
        InternalDocs::Disabled,
        AppState::new(Arc::new(FakeTokenVerifier::default())),
    )
    .oneshot(request)
    .await
    .expect("Given the router's infallible service, calling it should always succeed")
}

async fn body_json(response: Response) -> Value {
    let body = response
        .into_body()
        .collect()
        .await
        .expect("Given a fully buffered JSON body, collecting it should always succeed")
        .to_bytes();
    serde_json::from_slice(&body)
        .expect("Given the JSON body of the concepts handler, parsing it should always succeed")
}

#[tokio::test]
async fn should_answer_200_for_concepts_request() {
    let expected_result = StatusCode::OK;

    let result = get_concepts().await.status();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_answer_null_next_cursor_for_concepts_request() {
    let expected_result = Value::Null;

    let body = body_json(get_concepts().await).await;
    let result = body["next_cursor"].clone();

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn should_list_one_entry_per_mapped_concept_for_concepts_request() {
    // `json!` nests one macro call per token, so one call per entry keeps the depth constant
    // as the list grows.
    let every_mapped_concept = Value::Array(
        [
            ("revenue", "Revenue"),
            ("operating_income", "Operating Income"),
            ("net_income", "Net Income"),
            ("assets", "Assets"),
            ("liabilities", "Liabilities"),
            ("equity", "Equity"),
            ("operating_cash_flow", "Operating Cash Flow"),
            ("investing_cash_flow", "Investing Cash Flow"),
            ("financing_cash_flow", "Financing Cash Flow"),
            ("current_assets", "Current Assets"),
            ("non_current_assets", "Non-Current Assets"),
            ("cash_and_equivalents", "Cash and Equivalents"),
            ("short_term_investments", "Short-Term Investments"),
            ("accounts_receivable", "Accounts Receivable"),
            ("inventory", "Inventory"),
            ("property_plant_equipment", "Property, Plant & Equipment"),
            ("goodwill", "Goodwill"),
            ("intangible_assets", "Intangible Assets"),
            ("liabilities_and_equity", "Liabilities and Equity"),
            ("temporary_equity", "Temporary Equity"),
            ("current_liabilities", "Current Liabilities"),
            ("non_current_liabilities", "Non-Current Liabilities"),
            ("accounts_payable", "Accounts Payable"),
            ("short_term_debt", "Short-Term Debt"),
            ("long_term_debt", "Long-Term Debt"),
            (
                "equity_attributable_to_parent",
                "Equity Attributable to Parent",
            ),
            ("noncontrolling_interest", "Noncontrolling Interest"),
            ("additional_paid_in_capital", "Additional Paid-In Capital"),
            ("retained_earnings", "Retained Earnings"),
            ("treasury_stock", "Treasury Stock"),
            (
                "accumulated_other_comprehensive_income",
                "Accumulated Other Comprehensive Income",
            ),
            ("cost_of_revenue", "Cost of Revenue"),
            ("gross_profit", "Gross Profit"),
            ("selling_general_admin", "SG&A"),
            ("research_development", "R&D"),
            ("interest_expense", "Interest Expense"),
            ("operating_expenses", "Operating Expenses"),
            ("pretax_income", "Pretax Income"),
            ("income_tax_expense", "Income Tax Expense"),
            (
                "net_income_attributable_to_parent",
                "Net Income Attributable to Parent",
            ),
            (
                "net_income_attributable_to_noncontrolling_interest",
                "Net Income Attributable to Noncontrolling Interest",
            ),
            ("earnings_per_share_basic", "EPS (Basic)"),
            ("earnings_per_share_diluted", "EPS (Diluted)"),
            (
                "long_term_debt_including_current_maturities",
                "Long-Term Debt Including Current Maturities",
            ),
            ("debt_due_in_year_one", "Debt Due in Year One"),
            ("debt_due_in_year_two", "Debt Due in Year Two"),
            ("debt_due_in_year_three", "Debt Due in Year Three"),
            ("debt_due_in_year_four", "Debt Due in Year Four"),
            ("debt_due_in_year_five", "Debt Due in Year Five"),
            ("debt_due_after_year_five", "Debt Due After Year Five"),
            ("depreciation_amortization", "Depreciation & Amortization"),
            ("capital_expenditures", "Capital Expenditures"),
            ("dividends_paid", "Dividends Paid"),
            ("share_repurchases", "Share Repurchases"),
            ("share_based_compensation", "Share-Based Compensation"),
            ("net_change_in_cash", "Net Change in Cash"),
            ("exchange_rate_effect", "Exchange Rate Effect"),
            ("other_comprehensive_income", "Other Comprehensive Income"),
            ("comprehensive_income", "Comprehensive Income"),
        ]
        .iter()
        .map(|(concept, name)| json!({ "concept": concept, "name": name }))
        .collect(),
    );

    let expected_result = every_mapped_concept;

    let body = body_json(get_concepts().await).await;
    let result = body["data"].clone();

    assert_eq!(result, expected_result);
}
