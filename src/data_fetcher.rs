use serde::Deserialize;
use std::error::Error;
use serde_json::Value;

// Structure for cryptocurrency price data
#[derive(Debug, Deserialize, Clone)]
pub struct CryptoData {
    pub prices: Vec<(f64, f64)>,         // Timestamp and price pairs
    #[serde(default)]
    pub volumes: Vec<(f64, f64)>,        // Timestamp and volume pairs
    #[serde(default)]
    pub high_prices: Vec<(f64, f64)>,    // Timestamp and high price pairs
    #[serde(default)]
    pub low_prices: Vec<(f64, f64)>,     // Timestamp and low price pairs
    #[serde(default)]
    #[allow(dead_code)]
    pub open_prices: Vec<(f64, f64)>,    // Timestamp and open price pairs
    #[serde(default)]
    pub ohlc_data: Vec<(f64, f64, f64, f64, f64, f64)>, // Timestamp, open, high, low, close, volume
}

#[derive(Debug, Deserialize)]
struct FearGreedResponse {
    data: Vec<FearGreedData>,
    metadata: FearGreedMetadata,
}

#[derive(Debug, Deserialize)]
pub struct FearGreedData {
    pub value: String,
    pub value_classification: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
struct FearGreedMetadata {
    error: Option<String>,
}


/// Fetch Bitcoin price data from Binance API
async fn fetch_bitcoin_data(days: u32) -> Result<CryptoData, Box<dyn Error>> {
    // Calculate the start time (current time - days in milliseconds)
    let end_time = chrono::Utc::now().timestamp_millis() as u64;
    let start_time = end_time - (days as u64 * 24 * 60 * 60 * 1000);
    
    println!("Fetching data from {} to {}", 
        chrono::DateTime::<chrono::Utc>::from_timestamp((start_time / 1000) as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"),
        chrono::DateTime::<chrono::Utc>::from_timestamp((end_time / 1000) as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
    
    // Binance API endpoint - BTCUSDT 4h candles with explicit limit
    let url = format!(
        "https://api-gcp.binance.com/api/v3/klines?symbol=BTCUSDT&interval=4h&startTime={}&endTime={}&limit=1000",
        start_time, end_time
    );
    
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    if response.status().is_success() {
        let klines: Vec<Vec<Value>> = response.json().await?;
        println!("Retrieved {} candles in first request", klines.len());
        
        // If we got the maximum number of candles (1000) and need more,
        // perform additional requests to get the complete dataset
        let mut all_klines = klines;
        
        if all_klines.len() == 1000 {
            // We need to make additional requests
            // Get the timestamp of the last candle we received
            if let Some(last_candle) = all_klines.last() {
                if last_candle.len() > 6 {
                    // Use the close time (index 6) of the last candle as the new startTime
                    // Add 1 millisecond to avoid duplicating the last candle
                    let mut new_start_time = parse_to_f64(&last_candle[6]) as u64 + 1;
                    
                    // Keep fetching until we reach the end time or get no new data
                    let mut request_count = 1;
                    while new_start_time < end_time {
                        let pagination_url = format!(
                            "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=4h&startTime={}&endTime={}&limit=1000",
                            new_start_time, end_time
                        );
                        
                        let pagination_response = client.get(&pagination_url).send().await?;
                        
                        if pagination_response.status().is_success() {
                            let additional_klines: Vec<Vec<Value>> = pagination_response.json().await?;
                            println!("Pagination request {}: Retrieved {} additional candles", 
                                request_count, additional_klines.len());
                            
                            // If we got no new data, break the loop
                            if additional_klines.is_empty() {
                                break;
                            }
                            
                            // Update the start time for the next request
                            if let Some(next_last_candle) = additional_klines.last() {
                                if next_last_candle.len() > 6 {
                                    new_start_time = parse_to_f64(&next_last_candle[6]) as u64 + 1;
                                } else {
                                    break; // Incomplete candle data
                                }
                            } else {
                                break; // No more candles
                            }
                            
                            // Append the new data
                            all_klines.extend(additional_klines);
                            request_count += 1;
                        } else {
                            // If request failed, just use what we have
                            println!("Pagination request {} failed with status: {}", 
                                request_count, pagination_response.status());
                            break;
                        }
                    }
                }
            }
        }
        
        // Sort the data by timestamp to ensure chronological order
        all_klines.sort_by(|a, b| {
            if a.len() > 0 && b.len() > 0 {
                let time_a = parse_to_f64(&a[0]);
                let time_b = parse_to_f64(&b[0]);
                time_a.partial_cmp(&time_b).unwrap()
            } else {
                std::cmp::Ordering::Equal
            }
        });
        
        let data = convert_binance_data(all_klines);
        
        // Print the time range of the retrieved data
        if !data.prices.is_empty() {
            let first_timestamp = data.prices.first().unwrap().0;
            let last_timestamp = data.prices.last().unwrap().0;
            
            println!("Data retrieved from {} to {}", 
                chrono::DateTime::<chrono::Utc>::from_timestamp((first_timestamp / 1000.0) as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"),
                chrono::DateTime::<chrono::Utc>::from_timestamp((last_timestamp / 1000.0) as i64, 0).unwrap().format("%Y-%m-%d %H:%M:%S"));
            println!("Total candles: {}", data.prices.len());
        }
        
        Ok(data)
    } else {
        Err(format!("API request failed with status: {}", response.status()).into())
    }
}

// Helper function to safely parse a JSON value to f64
fn parse_to_f64(value: &Value) -> f64 {
    match value {
        Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        Value::Number(n) => n.as_f64().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Convert Binance API response to our CryptoData structure
fn convert_binance_data(klines: Vec<Vec<Value>>) -> CryptoData {
    let mut prices = Vec::new();
    let mut volumes = Vec::new();
    let mut high_prices = Vec::new();
    let mut low_prices = Vec::new();
    let mut open_prices = Vec::new();
    let mut ohlc_data = Vec::new();

    for kline in klines {
        if kline.len() >= 6 {
            // Parse values from the Binance kline response
            // [0] = Open time, [1] = Open, [2] = High, [3] = Low, [4] = Close, [5] = Volume
            let open_time = parse_to_f64(&kline[0]);
            let open = parse_to_f64(&kline[1]);
            let high = parse_to_f64(&kline[2]);
            let low = parse_to_f64(&kline[3]);
            let close = parse_to_f64(&kline[4]);
            let volume = parse_to_f64(&kline[5]);

            // Store all the data
            prices.push((open_time, close));
            volumes.push((open_time, volume));
            high_prices.push((open_time, high));
            low_prices.push((open_time, low));
            open_prices.push((open_time, open));
            ohlc_data.push((open_time, open, high, low, close, volume));
        }
    }

    CryptoData {
        prices,
        volumes,
        high_prices,
        low_prices,
        open_prices,
        ohlc_data,
    }
}

async fn fetch_fear_greed_index(limit: i32) -> Result<FearGreedResponse, Box<dyn Error>> {
    // Fetch the Fear & Greed Index data from the API
    let url = format!("https://api.alternative.me/fng/?limit={}", limit);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    
    if response.status().is_success() {
        let data: FearGreedResponse = response.json().await?;
        Ok(data)
    } else {
        Err(format!("API request failed with status: {}", response.status()).into())
    }
}

pub async fn fetch_fear_greed_index_data() -> Result<Vec<FearGreedData>, Box<dyn Error>> {
    // Fetch the latest Fear & Greed Index data
    match fetch_fear_greed_index(4).await {
        Ok(data) => {
            if data.metadata.error.is_some() {
                Err(format!("Error fetching Fear & Greed Index: {}", data.metadata.error.unwrap()).into())
            } else {
                Ok(data.data)
            }
        },
        Err(e) => Err(format!("Error fetching Fear & Greed Index: {}", e).into()),
    }
}
/// Fetch Bitcoin price data for a 4-month period with 4-hour candles
pub async fn fetch_bitcoin_trading_data() -> Result<CryptoData, Box<dyn Error>> {
    // 4 months = 120 days
    fetch_bitcoin_data(120).await
}