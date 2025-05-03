# Crypto-Forecast

A Rust-based cryptocurrency forecasting tool that analyzes Bitcoin market data and generates trading recommendations using AI.

## Features

- Fetches real-time Bitcoin price data from Binance API
- Retrieves Fear & Greed Index for market sentiment analysis
- Performs comprehensive technical analysis with various indicators:
  - Simple Moving Averages (SMA)
  - Exponential Moving Averages (EMA)
  - Relative Strength Index (RSI)
  - Moving Average Convergence Divergence (MACD)
  - Bollinger Bands
  - On Balance Volume (OBV)
  - Average True Range (ATR)
  - Support and resistance levels
- Generates AI-powered trading recommendations using Anthropic's Claude model
- Displays detailed Bitcoin trading analysis including price predictions, buy/sell positions, and risk assessment

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (2021 edition or later)
- An [Anthropic API key](https://console.anthropic.com/) for Claude AI access

### Setup

1. Clone the repository:
   ```
   git clone https://github.com/dumpsayamrat/crypto-forecast.git
   cd crypto-forecast
   ```

2. Create a `.env` file in the project root and add your Anthropic API key:
   ```
   ANTHROPIC_API_KEY=your_anthropic_api_key_here
   ```

3. Build the project:
   ```
   cargo build --release
   ```

## Usage

Run the application using Cargo:

```
cargo run
```

Or use the compiled binary directly:

```
./target/release/crypto-forecast
```

The application will:
1. Fetch the latest Bitcoin market data
2. Perform technical analysis
3. Generate a forecast using Claude AI
4. Display the results in the terminal

## Project Structure

- `src/main.rs`: Entry point and application flow coordinator
- `src/data_fetcher.rs`: Handles API requests to get market data
- `src/technical_analysis.rs`: Calculates technical indicators
- `src/prompt_generator.rs`: Creates prompts for the AI model
- `src/ai_client.rs`: Manages communication with the Claude API

## Dependencies

- `reqwest`: HTTP client for API requests
- `tokio`: Asynchronous runtime
- `serde` and `serde_json`: JSON serialization/deserialization
- `ta`: Technical analysis library for financial indicators
- `dotenv`: Environment variable management

## License

This project is licensed under the [MIT License](LICENSE).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Disclaimer

This tool is for informational purposes only. The generated trading recommendations should not be considered financial advice. Always conduct your own research before making investment decisions.