use std::env;
use std::error::Error;
use reqwest::Client;
use chrono::Utc;
use serde_json::json;

/// Output handler for different destinations
pub async fn send_output(analysis: &str, output_format: &str) -> Result<(), Box<dyn Error>> {
    match output_format {
        "telegram" => send_to_telegram(analysis).await,
        _ => {
            // Default text output with headers
            println!("\n=== BITCOIN TRADING RECOMMENDATIONS ===\n");
            println!("{}", analysis);
            println!("\n===============================");
            Ok(())
        }
    }
}

/// Send messages to Telegram in chunks to handle message size limits
async fn send_to_telegram(analysis: &str) -> Result<(), Box<dyn Error>> {
    // Get Telegram API key and chat ID from environment variables
    let telegram_api_key = env::var("TELEGRAM_API_KEY")
        .expect("TELEGRAM_API_KEY must be set when using telegram output format");
    let telegram_chat_id = env::var("TELEGRAM_CHAT_ID")
        .expect("TELEGRAM_CHAT_ID must be set when using telegram output format");
    
    // Create a reqwest client
    let client = Client::new();
    
    // Get current date/time for the header
    let date = Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    
    // Format header message
    let header = format!("📊 *Bitcoin Trading Analysis - {}*", date);
    
    // Send header first
    let header_url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        telegram_api_key
    );
    
    let header_payload = json!({
        "chat_id": telegram_chat_id,
        "text": header,
        "parse_mode": "Markdown"
    });
    
    let _ = client
        .post(&header_url)
        .json(&header_payload)
        .send()
        .await?;
    
    // Split analysis into chunks (Telegram has a 4096 character limit)
    let max_chunk_length = 3900;
    
    // Create an iterator over analysis that breaks it into chunks
    let mut position = 0;
    let total_length = analysis.len();
    
    while position < total_length {
        let remaining = total_length - position;
        let current_chunk_size = if remaining < max_chunk_length {
            remaining
        } else {
            // Try to find a good break point (newline)
            let potential_chunk = &analysis[position..position + max_chunk_length.min(remaining)];
            
            // Find the last newline in this potential chunk
            let last_newline_pos = potential_chunk.rfind('\n').unwrap_or(0);
            
            if last_newline_pos > 100 {
                // Break at the newline if it's not too close to the start
                last_newline_pos
            } else {
                // Otherwise use the maximum chunk size
                max_chunk_length.min(remaining)
            }
        };
        
        // Extract the chunk
        let chunk = &analysis[position..position + current_chunk_size];
        
        // Send this chunk
        let message_url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            telegram_api_key
        );
        
        let message_payload = json!({
            "chat_id": telegram_chat_id,
            "text": chunk,
            "parse_mode": "Markdown"
        });
        
        let _ = client
            .post(&message_url)
            .json(&message_payload)
            .send()
            .await?;
        
        // Move to next chunk
        position += current_chunk_size;
        
        // Add a small delay to avoid rate limiting
        if position < total_length {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
    
    // Print a confirmation message to stdout
    println!("Analysis sent to Telegram successfully!");
    
    Ok(())
}
