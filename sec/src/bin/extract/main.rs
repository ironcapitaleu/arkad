mod extraction;

use extraction::Extraction;
use futures_util::StreamExt;

const CIKS: [&str; 62] = [
    "ABC",        // Invalid CIK - non-n
    "9999999999", // Nonsense CIK
    "1067983",    // Berkshire Hathaway
    "320193",     // Apple
    "789019",     // Microsoft
    "1018724",    // Amazon
    "1652044",    // Alphabet (Google)
    "1326801",    // Meta (Facebook)
    "1318605",    // Tesla
    "1730168",    // Nvidia
    "78003",      // Pfizer
    "200406",     // Johnson & Johnson
    "21344",      // Coca-Cola
    "12927",      // Chevron
    "34088",      // Exxon Mobil
    "93410",      // Caterpillar
    "51143",      // IBM
    "732717",     // Wells Fargo
    "70858",      // Bank of America
    "19617",      // JPMorgan Chase
    "831001",     // Citigroup
    "886982",     // Goldman Sachs
    "895421",     // Morgan Stanley
    "18230",      // Charles Schwab
    "927628",     // T-Mobile
    "1283699",    // Costco
    "354950",     // Home Depot
    "60667",      // Lockheed Martin
    "40545",      // General Electric
    "310158",     // Merck
    "14272",      // Bristol-Myers Squibb
    "4962",       // AbbVie
    "1551152",    // Salesforce
    "1045810",    // Netflix
    "1403161",    // Visa
    "1141391",    // Mastercard
    "2488",       // AMD
    "1413329",    // PayPal
    "97476",      // Texas Instruments
    "804328",     // Qualcomm
    "1090727",    // UnitedHealth
    "1800",       // Abbott Labs
    "80424",      // PepsiCo
    "77476",      // Procter & Gamble
    "1373715",    // Uber
    "1364742",    // Booking Holdings
    "24741",      // ConocoPhillips
    "50863",      // Intel
    "66740",      // 3M
    "731766",     // United Parcel Service
    "1166126",    // Starbucks
    "1555280",    // Airbnb
    "1564708",    // ServiceNow
    "896159",     // Lowe's
    "858877",     // Cisco
    "27419",      // Deere & Co
    "37996",      // Ford Motor
    "1467858",    // Block (Square)
    "1679788",    // CrowdStrike
    "1386278",    // Palo Alto Networks
    "718877",     // Raytheon (RTX)
    "1324404",    // Palantir
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let start = std::time::Instant::now();

    let results: Vec<_> = futures_util::stream::iter(CIKS)
        .map(|cik| Extraction::builder().cik(cik).build().run())
        .buffer_unordered(10)
        .collect()
        .await;

    let elapsed = start.elapsed();

    let mut successes = 0;
    let mut failures = 0;
    for result in &results {
        match result {
            Ok(()) => successes += 1,
            Err(e) => {
                failures += 1;
                eprintln!("Error: {e}");
            }
        }
    }

    println!("\n{successes} succeeded, {failures} failed in {elapsed:.2?}");

    Ok(())
}
