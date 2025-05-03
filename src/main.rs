mod data_fetcher;
mod technical_analysis;
mod prompt_generator;
mod ai_client;

use dotenv::dotenv;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Check for command-line arguments
    let args: Vec<String> = env::args().collect();
    let only_prompt = args.len() > 1 && args[1] == "--only-prompt";
    
    // Get Anthropic API key from environment variables (only if we need it)
    let api_key = if !only_prompt {
        env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY must be set in the .env file")
    } else {
        String::new() // Dummy value since we won't be using it
    };
    
    println!("Fetching Bitcoin price data from Binance...");
    
    // Get Bitcoin price data for trading analysis (4-hour candles over 4 months)
    let btc_data = data_fetcher::fetch_bitcoin_trading_data().await?;
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
        println!("\n===============================");
    } else {
        // Get analysis from Claude
        let analysis = ai_client::get_analysis_from_claude(&api_key, &prompt).await?;
        
        // Display the analysis
        println!("\n=== BITCOIN TRADING RECOMMENDATIONS ===\n");
        println!("{}", analysis);
        println!("\n===============================");
    }
    
    Ok(())
}
