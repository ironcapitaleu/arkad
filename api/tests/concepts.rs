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
    api::router()
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
    let every_mapped_concept = json!([
        { "concept": "revenue", "name": "Revenue" },
        { "concept": "operating_income", "name": "Operating Income" },
        { "concept": "net_income", "name": "Net Income" },
        { "concept": "assets", "name": "Assets" },
        { "concept": "liabilities", "name": "Liabilities" },
        { "concept": "equity", "name": "Equity" },
        { "concept": "operating_cash_flow", "name": "Operating Cash Flow" },
        { "concept": "investing_cash_flow", "name": "Investing Cash Flow" },
        { "concept": "financing_cash_flow", "name": "Financing Cash Flow" },
        { "concept": "current_assets", "name": "Current Assets" },
        { "concept": "non_current_assets", "name": "Non-Current Assets" },
        { "concept": "cash_and_equivalents", "name": "Cash and Equivalents" },
        { "concept": "short_term_investments", "name": "Short-Term Investments" },
        { "concept": "accounts_receivable", "name": "Accounts Receivable" },
        { "concept": "inventory", "name": "Inventory" },
        { "concept": "property_plant_equipment", "name": "Property, Plant & Equipment" },
        { "concept": "goodwill", "name": "Goodwill" },
        { "concept": "intangible_assets", "name": "Intangible Assets" },
        { "concept": "current_liabilities", "name": "Current Liabilities" },
        { "concept": "non_current_liabilities", "name": "Non-Current Liabilities" },
        { "concept": "long_term_debt", "name": "Long-Term Debt" },
        { "concept": "retained_earnings", "name": "Retained Earnings" },
        { "concept": "cost_of_revenue", "name": "Cost of Revenue" },
        { "concept": "gross_profit", "name": "Gross Profit" },
        { "concept": "selling_general_admin", "name": "SG&A" },
        { "concept": "research_development", "name": "R&D" },
        { "concept": "interest_expense", "name": "Interest Expense" },
        { "concept": "earnings_per_share_basic", "name": "EPS (Basic)" },
        { "concept": "earnings_per_share_diluted", "name": "EPS (Diluted)" },
        { "concept": "depreciation_amortization", "name": "Depreciation & Amortization" },
        { "concept": "capital_expenditures", "name": "Capital Expenditures" },
        { "concept": "dividends_paid", "name": "Dividends Paid" },
        { "concept": "share_repurchases", "name": "Share Repurchases" },
        { "concept": "share_based_compensation", "name": "Share-Based Compensation" },
        { "concept": "other_comprehensive_income", "name": "Other Comprehensive Income" },
        { "concept": "comprehensive_income", "name": "Comprehensive Income" },
    ]);

    let expected_result = every_mapped_concept;

    let body = body_json(get_concepts().await).await;
    let result = body["data"].clone();

    assert_eq!(result, expected_result);
}
