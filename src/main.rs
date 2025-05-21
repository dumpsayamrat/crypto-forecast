mod data_fetcher;
mod technical_analysis;
mod prompt_generator;
mod ai_client;
mod output;

use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Check for command-line arguments
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments
    let mut output_format = "text";
    let mut only_prompt = false;
    
    if args.len() > 1 {
        if args[1] == "--only-prompt" {
            only_prompt = true;
        } else {
            output_format = &args[1];
        }
    }
    
    // Get Anthropic API key from environment variables (only if we need it)
    let api_key = if !only_prompt {
        env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY must be set in the .env file")
    } else {
        String::new()
    };

    let data_provider_api_key = env::var("DATA_PROVIDER_API_KEY")
        .unwrap_or_else(|_| String::new());
    
    let api_base_url = env::var("API_BASE_URL")
        .unwrap_or_else(|_| "https://api.binance.com".to_string());
    
    println!("Fetching Bitcoin price data from API...");
    
    // Get Bitcoin price data for trading analysis (4-hour candles over 4 months)
    let btc_data = data_fetcher::fetch_bitcoin_trading_data(&data_provider_api_key, &api_base_url).await?;
    let fear_and_greed_data = data_fetcher::fetch_fear_greed_index_data().await?;

    println!("Analyzing Bitcoin price data with RSI(14), MACD(12,26,9), and other indicators...");
    
    // Prepare the data for analysis, including technical indicators
    let formatted_data = technical_analysis::format_data_for_analysis(&btc_data, &fear_and_greed_data);
    
    // Generate trading recommendations prompt by default
    println!("\nGenerating trading recommendations...");
    let prompt = prompt_generator::generate_trading_recommendation_prompt(&formatted_data);
    
    if only_prompt {
        // Display only the prompt
        println!("\n=== PROMPT ===\n");
        println!("{}", prompt);
        println!("\n===============================");    } else {        // Get analysis from Claude
        let analysis = ai_client::get_analysis_from_claude(&api_key, &prompt).await?;
        
        // Use the output module to handle the output formatting
        output::send_output(&analysis, output_format).await?;    }
    
    Ok(())
}
