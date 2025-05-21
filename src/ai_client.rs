use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::error::Error;

// Structure for Anthropic API requests
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
pub struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Debug, Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

// Structure for Anthropic API responses
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ResponseContent>,
}

#[derive(Debug, Deserialize)]
struct ResponseContent {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    content_type: String,
    text: String,
}

/// Get analysis from Anthropic Claude API
pub async fn get_analysis_from_claude(api_key: &str, prompt: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    
    // Set up headers
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(api_key)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    
    // Prepare the request body - increase max_tokens to 4096 (Claude's maximum)
    let request_body = AnthropicRequest {
        model: "claude-3-7-sonnet-20250219".to_string(),
        max_tokens: 4096,
        messages: vec![Message {
            role: "user".to_string(),
            content: vec![Content {
                content_type: "text".to_string(),
                text: prompt.to_string(),
            }],
        }],
    };
    
    // Send the request
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;
    
    if response.status().is_success() {
        let response_data: AnthropicResponse = response.json().await?;
          // Extract the prediction text
        if let Some(content) = response_data.content.first() {
            // Extract the market analysis from the response if it contains <bitcoin_market_analysis> tags
            let market_analysis = extract_bitcoin_market_analysis(&content.text);
            
            // Extract the last 3 data points from the prompt
            let last_3_data_points = extract_last_3_data_points(prompt);
            
            // Combine the Claude response with the data points
            let mut final_response = String::new();
            
            // Add the last 3 data points section
            final_response.push_str("=== LAST 3 DATA POINTS ===\n");
            final_response.push_str(&last_3_data_points);
            final_response.push_str("\n\n");
            
            // Add the Claude response (either the structured analysis or the full response)
            final_response.push_str("=== BITCOIN MARKET ANALYSIS ===\n");
            final_response.push_str(&market_analysis);
            
            Ok(final_response)
        } else {
            Err("No content in the response".into())
        }
    } else {
        Err(format!("API request failed with status: {}", response.status()).into())
    }
}

/// Extract the last 3 data points from the prompt
fn extract_last_3_data_points(prompt: &str) -> String {
    let mut last_3_lines = String::new();
    
    // Check for the new format with <historical_data> tags
    if let Some(hist_start) = prompt.find("<historical_data>") {
        let data_start = hist_start + "<historical_data>".len();
        
        if let Some(hist_end) = prompt[data_start..].find("</historical_data>") {
            let historical_data = &prompt[data_start..(data_start + hist_end)].trim();
            
            // Parse the historical data section to find OHLCV data
            let data_lines: Vec<&str> = historical_data
                .lines()
                .filter(|line| line.contains("O=$") || line.contains("Price=$") || line.contains(": C=$"))
                .collect();
            
            // Get the last 3 lines if available
            let start_idx = if data_lines.len() >= 3 {
                data_lines.len() - 3
            } else {
                0
            };
            
            for i in start_idx..data_lines.len() {
                last_3_lines.push_str(data_lines[i]);
                last_3_lines.push_str("\n");
            }
        }
    } 
    
    // Fallback to the old format if the new format is not found
    if last_3_lines.is_empty() {
        if let Some(ohlc_start) = prompt.find("Bitcoin historical OHLC + Volume data") {
            // Find where the data starts (usually after "Date,Open,High,Low,Close,Volume")
            if let Some(data_start) = prompt[ohlc_start..].find("\n") {
                let data_section = &prompt[(ohlc_start + data_start + 1)..];
                
                // Find where the data ends (usually before "=== TECHNICAL INDICATORS ===")
                let data_end = if let Some(end_idx) = data_section.find("===") {
                    end_idx
                } else {
                    data_section.len()
                };
                
                let data_lines: Vec<&str> = data_section[..data_end]
                    .trim()
                    .lines()
                    .collect();
                
                // Get the last 3 lines if available
                let start_idx = if data_lines.len() >= 3 {
                    data_lines.len() - 3
                } else {
                    0
                };
                
                for i in start_idx..data_lines.len() {
                    last_3_lines.push_str(data_lines[i]);
                    last_3_lines.push_str("\n");
                }
            }
        } else if let Some(price_start) = prompt.find("Bitcoin price data (timestamp, price in USD)") {
            if let Some(data_start) = prompt[price_start..].find("\n") {
                let data_section = &prompt[(price_start + data_start + 1)..];
                
                // Find where the data ends
                let data_end = if let Some(end_idx) = data_section.find("===") {
                    end_idx
                } else {
                    data_section.len()
                };
                
                let data_lines: Vec<&str> = data_section[..data_end]
                    .trim()
                    .lines()
                    .collect();
                
                // Get the last 3 lines if available
                let start_idx = if data_lines.len() >= 3 {
                    data_lines.len() - 3
                } else {
                    0
                };
                
                for i in start_idx..data_lines.len() {
                    last_3_lines.push_str(data_lines[i]);
                    last_3_lines.push_str("\n");
                }
            }
        }
    }
    
    if last_3_lines.is_empty() {
        "No data points found in the prompt.".to_string()
    } else {
        last_3_lines
    }
}

/// Extract the Bitcoin market analysis from the AI's response
fn extract_bitcoin_market_analysis(response: &str) -> String {
    // Look for content within <bitcoin_market_analysis> tags
    if let Some(start_idx) = response.find("<bitcoin_market_analysis>") {
        let content_start = start_idx + "<bitcoin_market_analysis>".len();
        
        if let Some(end_idx) = response[content_start..].find("</bitcoin_market_analysis>") {
            // Return just the content within the tags
            return response[content_start..(content_start + end_idx)].trim().to_string();
        }
    }
    
    // If no tags found or format is incorrect, return the full response
    response.to_string()
}