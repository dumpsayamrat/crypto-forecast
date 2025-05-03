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
            // Extract the last 3 data points from the prompt
            let last_3_data_points = extract_last_3_data_points(prompt);
            let ta_summary = extract_ta_summary(prompt);
            let fear_greed_data = extract_fear_greed(prompt);
            
            // Combine the Claude response with the data points
            let mut final_response = String::new();
            
            // Add the last 3 data points section
            final_response.push_str("=== LAST 3 DATA POINTS ===\n");
            final_response.push_str(&last_3_data_points);
            final_response.push_str("\n\n");
            
            // Add the technical analysis summary
            final_response.push_str("=== TECHNICAL ANALYSIS SUMMARY ===\n");
            final_response.push_str(&ta_summary);
            final_response.push_str("\n\n");
            
            // Add the fear and greed data
            final_response.push_str("=== FEAR AND GREED INDEX ===\n");
            final_response.push_str(&fear_greed_data);
            final_response.push_str("\n\n");
            
            // Add the Claude response
            final_response.push_str("=== CLAUDE ANALYSIS ===\n");
            final_response.push_str(&content.text);
            
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
    
    // Find the OHLC data section
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
    } else {
        // Fallback if OHLC data not found - look for basic price data
        if let Some(price_start) = prompt.find("Bitcoin price data (timestamp, price in USD)") {
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

/// Extract the technical analysis summary from the prompt
fn extract_ta_summary(prompt: &str) -> String {
    let mut ta_summary = String::new();
    
    // Extract key indicators
    let indicators = [
        "Simple Moving Averages:", 
        "Exponential Moving Averages:", 
        "RSI", 
        "MACD", 
        "Bollinger Bands", 
        "On Balance Volume", 
        "Average True Range"
    ];
    
    if let Some(ta_start) = prompt.find("=== TECHNICAL INDICATORS ===") {
        let ta_section = &prompt[ta_start..];
        
        for indicator in &indicators {
            if let Some(ind_start) = ta_section.find(indicator) {
                let ind_section = &ta_section[ind_start..];
                let ind_end = if let Some(end_idx) = ind_section[1..].find("\n\n") {
                    end_idx + 1
                } else {
                    ind_section.len().min(200) // Limit length if no clear end
                };
                
                // Extract just the first few lines of each indicator section
                let ind_text = &ind_section[..ind_end];
                let ind_lines: Vec<&str> = ind_text.lines().take(3).collect();
                
                for line in ind_lines {
                    ta_summary.push_str(line);
                    ta_summary.push_str("\n");
                }
                
                ta_summary.push_str("\n");
            }
        }
        
        // Add support and resistance levels
        if let Some(support_start) = ta_section.find("Support level:") {
            let support_text = &ta_section[support_start..];
            let support_end = if let Some(end_idx) = support_text.find("\n") {
                end_idx
            } else {
                support_text.len()
            };
            
            ta_summary.push_str(&support_text[..support_end]);
            ta_summary.push_str("\n");
        }
        
        if let Some(resistance_start) = ta_section.find("Resistance level:") {
            let resistance_text = &ta_section[resistance_start..];
            let resistance_end = if let Some(end_idx) = resistance_text.find("\n") {
                end_idx
            } else {
                resistance_text.len()
            };
            
            ta_summary.push_str(&resistance_text[..resistance_end]);
            ta_summary.push_str("\n");
        }
    }
    
    if ta_summary.is_empty() {
        "No technical analysis found in the prompt.".to_string()
    } else {
        ta_summary
    }
}

/// Extract the fear and greed index data from the prompt
fn extract_fear_greed(prompt: &str) -> String {
    let mut fear_greed_data = String::new();
    
    // Look for the Fear and Greed section header (both formats)
    let section_headers = ["=== FEAR & GREED INDEX ===", "=== FEAR AND GREED INDEX ==="];
    
    for &header in &section_headers {
        if let Some(fg_start) = prompt.find(header) {
            // Skip the header line
            if let Some(line_end) = prompt[fg_start..].find('\n') {
                let content_start = fg_start + line_end + 1;
                
                // Take next 5 lines after the header
                let lines: Vec<&str> = prompt[content_start..]
                    .lines()
                    .take(5)
                    .collect();
                
                for line in lines {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("===") {
                        fear_greed_data.push_str(trimmed);
                        fear_greed_data.push_str("\n");
                    }
                }
                
                // If we found data, break out of the loop
                if !fear_greed_data.is_empty() {
                    break;
                }
            }
        }
    }
    
    if fear_greed_data.is_empty() {
        "No Fear and Greed data found in the prompt.".to_string()
    } else {
        fear_greed_data
    }
}