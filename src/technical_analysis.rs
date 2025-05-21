use chrono::{DateTime, Utc};
use crate::data_fetcher::{CryptoData, FearGreedData};
use ta::indicators::{
    MovingAverageConvergenceDivergence, RelativeStrengthIndex,
    ExponentialMovingAverage, SimpleMovingAverage, 
    BollingerBands, AverageTrueRange
};
use ta::Next;
use std::cmp::min;

/// Format Bitcoin data into a string for analysis, including technical indicators
pub fn format_data_for_analysis(data: &CryptoData, fng: &Vec<FearGreedData>) -> String {
    let mut formatted_data = String::new();
    
    // Check if OHLC data is available and non-empty
    if !data.ohlc_data.is_empty() {
        // Add a summary of historical data
        formatted_data.push_str("=== BITCOIN HISTORICAL DATA SUMMARY ===\n");
        
        // Create vectors to store prices for sorting
        let mut all_prices: Vec<(DateTime<Utc>, f64, f64, f64, f64)> = vec![];
        let mut all_volumes: Vec<(DateTime<Utc>, f64)> = vec![];
        
        for i in 0..data.ohlc_data.len() {
            let (timestamp, open, high, low, close, volume) = data.ohlc_data[i];
            let date = DateTime::<Utc>::from_timestamp((timestamp as i64) / 1000, 0).unwrap();
            all_prices.push((date, open, high, low, close));
            all_volumes.push((date, volume));
        }
        
        // Find 5 highest and 5 lowest closing prices
        let mut price_date_pairs: Vec<(DateTime<Utc>, f64)> = all_prices.iter()
            .map(|(date, _, _, _, close)| (*date, *close))
            .collect();
        
        price_date_pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        formatted_data.push_str("\n5 Highest Bitcoin Prices (All-Time):\n");
        for (i, (date, price)) in price_date_pairs.iter().take(5).enumerate() {
            formatted_data.push_str(&format!("{}. {}: ${:.2}\n", 
                i+1, date.format("%Y-%m-%d %H:%M:%S"), price));
        }
        
        price_date_pairs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        formatted_data.push_str("\n5 Lowest Bitcoin Prices (All-Time):\n");
        for (i, (date, price)) in price_date_pairs.iter().take(5).enumerate() {
            formatted_data.push_str(&format!("{}. {}: ${:.2}\n", 
                i+1, date.format("%Y-%m-%d %H:%M:%S"), price));
        }
        
        // Calculate some key statistics
        if !all_prices.is_empty() {
            let close_prices: Vec<f64> = all_prices.iter().map(|(_, _, _, _, close)| *close).collect();
            let high_prices: Vec<f64> = all_prices.iter().map(|(_, _, high, _, _)| *high).collect();
            let low_prices: Vec<f64> = all_prices.iter().map(|(_, _, _, low, _)| *low).collect();
            let volumes: Vec<f64> = all_volumes.iter().map(|(_, volume)| *volume).collect();
            
            let avg_close = close_prices.iter().sum::<f64>() / close_prices.len() as f64;
            let max_price = *high_prices.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);
            let min_price = *low_prices.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);
            let avg_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;
            
            // Calculate volatility (standard deviation of closing prices)
            let variance = close_prices.iter()
                .map(|price| {
                    let diff = price - avg_close;
                    diff * diff
                })
                .sum::<f64>() / close_prices.len() as f64;
            let std_dev = variance.sqrt();
            
            formatted_data.push_str("\nKey Statistics:\n");
            formatted_data.push_str(&format!("Average Price: ${:.2}\n", avg_close));
            formatted_data.push_str(&format!("All-Time High: ${:.2}\n", max_price));
            formatted_data.push_str(&format!("All-Time Low: ${:.2}\n", min_price));
            formatted_data.push_str(&format!("Price Range: ${:.2} (${:.2} to ${:.2})\n", max_price - min_price, min_price, max_price));
            formatted_data.push_str(&format!("Price Volatility (Std Dev): ${:.2} ({:.2}%)\n", std_dev, (std_dev / avg_close) * 100.0));
            formatted_data.push_str(&format!("Average Daily Volume: {:.2}\n", avg_volume));
            
            // Calculate price change over different periods
            if close_prices.len() >= 30 {
                let current_price = *close_prices.last().unwrap();
                let price_30_days_ago = close_prices[close_prices.len() - 30];
                let price_7_days_ago = close_prices[close_prices.len() - min(7, close_prices.len())];
                
                let change_30_days = (current_price - price_30_days_ago) / price_30_days_ago * 100.0;
                let change_7_days = (current_price - price_7_days_ago) / price_7_days_ago * 100.0;
                
                formatted_data.push_str(&format!("30-Day Price Change: {:.2}%\n", change_30_days));
                formatted_data.push_str(&format!("7-Day Price Change: {:.2}%\n", change_7_days));
            }
        }
        
        // Show recent data (last 24 records)
        formatted_data.push_str("\n=== RECENT BITCOIN OHLCV DATA (LAST 24 RECORDS) ===\n");
        formatted_data.push_str("Date,Open,High,Low,Close,Volume\n");
        
        // Get just the last 24 records
        let start_idx = if data.ohlc_data.len() > 24 { data.ohlc_data.len() - 24 } else { 0 };
        for i in start_idx..data.ohlc_data.len() {
            let (timestamp, open, high, low, close, volume) = data.ohlc_data[i];
            let date = DateTime::<Utc>::from_timestamp((timestamp as i64) / 1000, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            
            formatted_data.push_str(&format!("{}: O=${:.2} H=${:.2} L=${:.2} C=${:.2} V={:.2}\n", 
                date, open, high, low, close, volume));        }
    } else {
        // Add debug info to see why OHLC data might be empty
        formatted_data.push_str(&format!("Bitcoin price data (timestamp, price in USD): [Debug: OHLC data size: {}, Volumes size: {}]\n", 
            data.ohlc_data.len(), data.volumes.len()));
          
        // Fallback to basic price data if OHLC not available
        for (timestamp, price) in &data.prices {
            let date = DateTime::<Utc>::from_timestamp((*timestamp as i64) / 1000, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
                
            formatted_data.push_str(&format!("{}: Price=${:.2}\n", date, price));
        }
    }
    
    // Add technical indicators here
    formatted_data.push_str(&calculate_technical_indicators(data));
    
    // Add Fear & Greed Index data
    formatted_data.push_str(&format_fear_greed_data(fng));

    formatted_data
}

fn format_fear_greed_data(data: &Vec<FearGreedData>) -> String {
    let mut formatted_data = String::new();
    
    formatted_data.push_str("\n=== FEAR & GREED INDEX ===\n");
    formatted_data.push_str("Date: Index classification - Index value\n");
    
    for entry in data {
        let date = DateTime::<Utc>::from_timestamp(entry.timestamp.parse::<i64>().unwrap(), 0)
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();
        
        formatted_data.push_str(&format!("{}: {} - {}\n", date, entry.value_classification, entry.value));
    }
    
    formatted_data
}

/// Calculate technical indicators for Bitcoin price data
fn calculate_technical_indicators(data: &CryptoData) -> String {
    let mut result = String::new();
    
    // Extract just the prices for calculations
    let price_values: Vec<f64> = data.prices.iter().map(|(_, price)| *price).collect();
    
    // Calculate volume (if available)
    let volume_values: Vec<f64> = if !data.volumes.is_empty() {
        data.volumes.iter().map(|(_, volume)| *volume).collect()
    } else {
        vec![]
    };
    
    // Calculate high, low, close prices (if available)
    let high_values: Vec<f64> = if !data.high_prices.is_empty() {
        data.high_prices.iter().map(|(_, price)| *price).collect()
    } else {
        price_values.clone() // Use close price as fallback
    };
    
    let low_values: Vec<f64> = if !data.low_prices.is_empty() {
        data.low_prices.iter().map(|(_, price)| *price).collect()
    } else {
        price_values.clone() // Use close price as fallback
    };
    
    result.push_str("\n=== TECHNICAL INDICATORS ===\n");
      // Simple Moving Averages (SMA)
    if price_values.len() >= 200 {
        let mut sma7 = SimpleMovingAverage::new(7).unwrap();
        let mut sma20 = SimpleMovingAverage::new(20).unwrap();
        let mut sma50 = SimpleMovingAverage::new(50).unwrap();
        let mut sma200 = SimpleMovingAverage::new(200).unwrap();
        
        // Store last 5 values for each indicator
        let mut sma7_values = Vec::new();
        let mut sma20_values = Vec::new();
        let mut sma50_values = Vec::new();
        let mut sma200_values = Vec::new();
        let mut timestamps = Vec::new();
        
        // Get timestamps for the last 5 periods
        if data.prices.len() >= 5 {
            for i in (data.prices.len() - 5)..data.prices.len() {
                timestamps.push(data.prices[i].0);
            }
        }
        
        // Process all prices to calculate indicators
        for (i, &price) in price_values.iter().enumerate() {
            let sma7_val = sma7.next(price);
            let sma20_val = sma20.next(price);
            let sma50_val = sma50.next(price);
            let sma200_val = sma200.next(price);
            
            // Only keep the last 5 values
            if i >= price_values.len() - 5 {
                sma7_values.push(sma7_val);
                sma20_values.push(sma20_val);
                sma50_values.push(sma50_val);
                sma200_values.push(sma200_val);
            }
        }
        
        result.push_str("\nSimple Moving Averages (Last 5 periods):\n");
        
        // Display timestamps and SMA values for the last 5 periods
        for i in 0..min(5, sma7_values.len()) {
            let date = if i < timestamps.len() {
                DateTime::<Utc>::from_timestamp((timestamps[i] as i64) / 1000, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                format!("Period -{}", 5-i)
            };
            
            result.push_str(&format!("{}:\n", date));
            result.push_str(&format!("  SMA (7-period): ${:.2}\n", sma7_values[i]));
            result.push_str(&format!("  SMA (20-period): ${:.2}\n", sma20_values[i]));
            result.push_str(&format!("  SMA (50-period): ${:.2}\n", sma50_values[i]));
            result.push_str(&format!("  SMA (200-period): ${:.2}\n", sma200_values[i]));
        }
        
        // Add trend indications based on most recent SMA crossovers
        let most_recent_sma7 = *sma7_values.last().unwrap();
        let most_recent_sma20 = *sma20_values.last().unwrap();
        let most_recent_sma50 = *sma50_values.last().unwrap();
        let most_recent_sma200 = *sma200_values.last().unwrap();
        
        result.push_str("\nSMA Trend Analysis:\n");
        if most_recent_sma7 > most_recent_sma20 {
            result.push_str("Short-term Trend: Bullish (7 above 20)\n");
        } else {
            result.push_str("Short-term Trend: Bearish (7 below 20)\n");
        }
        
        if most_recent_sma50 > most_recent_sma200 {
            result.push_str("Long-term Trend: Bullish (50 above 200, Golden Cross active)\n");
        } else {
            result.push_str("Long-term Trend: Bearish (50 below 200, Death Cross active)\n");
        }
        
        // Price position relative to major SMAs
        let current_price = *price_values.last().unwrap();
        result.push_str(&format!("Price relative to SMAs: {}\n", 
            if current_price > most_recent_sma200 && current_price > most_recent_sma50 {
                "Strong bullish (Price above both 50 & 200 SMAs)"
            } else if current_price > most_recent_sma200 {
                "Moderately bullish (Price above 200 SMA but below 50 SMA)"
            } else if current_price > most_recent_sma50 {
                "Mixed signals (Price above 50 SMA but below 200 SMA)"
            } else {
                "Bearish (Price below both 50 & 200 SMAs)"
            }
        ));
    } else if price_values.len() >= 20 {
        let mut sma7 = SimpleMovingAverage::new(7).unwrap();
        let mut sma20 = SimpleMovingAverage::new(20).unwrap();
        
        // Store last 5 values for each indicator
        let mut sma7_values = Vec::new();
        let mut sma20_values = Vec::new();
        let mut timestamps = Vec::new();
        
        // Get timestamps for the last 5 periods
        if data.prices.len() >= 5 {
            for i in (data.prices.len() - 5)..data.prices.len() {
                timestamps.push(data.prices[i].0);
            }
        }
        
        // Process all prices to calculate indicators
        for (i, &price) in price_values.iter().enumerate() {
            let sma7_val = sma7.next(price);
            let sma20_val = sma20.next(price);
            
            // Only keep the last 5 values
            if i >= price_values.len() - 5 {
                sma7_values.push(sma7_val);
                sma20_values.push(sma20_val);
            }
        }
        
        result.push_str("\nSimple Moving Averages (Last 5 periods):\n");
        
        // Display timestamps and SMA values for the last 5 periods
        for i in 0..min(5, sma7_values.len()) {
            let date = if i < timestamps.len() {
                DateTime::<Utc>::from_timestamp((timestamps[i] as i64) / 1000, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                format!("Period -{}", 5-i)
            };
            
            result.push_str(&format!("{}:\n", date));
            result.push_str(&format!("  SMA (7-day): ${:.2}\n", sma7_values[i]));
            result.push_str(&format!("  SMA (20-day): ${:.2}\n", sma20_values[i]));
        }
        
        // Add trend indication based on SMA crossover
        let most_recent_sma7 = *sma7_values.last().unwrap();
        let most_recent_sma20 = *sma20_values.last().unwrap();
        
        result.push_str("\nSMA Trend Analysis:\n");
        if most_recent_sma7 > most_recent_sma20 {
            result.push_str("Trend: Bullish (Short-term SMA above Long-term SMA)\n");
        } else {
            result.push_str("Trend: Bearish (Short-term SMA below Long-term SMA)\n");
        }
    }
      // Exponential Moving Averages (EMA)
    if price_values.len() >= 200 {
        let mut ema12 = ExponentialMovingAverage::new(12).unwrap();
        let mut ema26 = ExponentialMovingAverage::new(26).unwrap();
        let mut ema50 = ExponentialMovingAverage::new(50).unwrap();
        let mut ema200 = ExponentialMovingAverage::new(200).unwrap();
        
        // Store last 5 values for each indicator
        let mut ema12_values = Vec::new();
        let mut ema26_values = Vec::new();
        let mut ema50_values = Vec::new();
        let mut ema200_values = Vec::new();
        let mut timestamps = Vec::new();
        
        // Get timestamps for the last 5 periods
        if data.prices.len() >= 5 {
            for i in (data.prices.len() - 5)..data.prices.len() {
                timestamps.push(data.prices[i].0);
            }
        }
        
        // Process all prices to calculate indicators
        for (i, &price) in price_values.iter().enumerate() {
            let ema12_val = ema12.next(price);
            let ema26_val = ema26.next(price);
            let ema50_val = ema50.next(price);
            let ema200_val = ema200.next(price);
            
            // Only keep the last 5 values
            if i >= price_values.len() - 5 {
                ema12_values.push(ema12_val);
                ema26_values.push(ema26_val);
                ema50_values.push(ema50_val);
                ema200_values.push(ema200_val);
            }
        }
        
        result.push_str("\nExponential Moving Averages (Last 5 periods):\n");
        
        // Display timestamps and EMA values for the last 5 periods
        for i in 0..min(5, ema12_values.len()) {
            let date = if i < timestamps.len() {
                DateTime::<Utc>::from_timestamp((timestamps[i] as i64) / 1000, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                format!("Period -{}", 5-i)
            };
            
            result.push_str(&format!("{}:\n", date));
            result.push_str(&format!("  EMA (12-period): ${:.2}\n", ema12_values[i]));
            result.push_str(&format!("  EMA (26-period): ${:.2}\n", ema26_values[i]));
            result.push_str(&format!("  EMA (50-period): ${:.2}\n", ema50_values[i]));
            result.push_str(&format!("  EMA (200-period): ${:.2}\n", ema200_values[i]));
        }
        
        // Add trend indications based on most recent EMA crossovers
        let most_recent_ema12 = *ema12_values.last().unwrap();
        let most_recent_ema26 = *ema26_values.last().unwrap();
        let most_recent_ema50 = *ema50_values.last().unwrap();
        let most_recent_ema200 = *ema200_values.last().unwrap();
        
        result.push_str("\nEMA Trend Analysis:\n");
        if most_recent_ema12 > most_recent_ema26 {
            result.push_str("Short-term EMA Trend: Bullish (12 above 26)\n");
        } else {
            result.push_str("Short-term EMA Trend: Bearish (12 below 26)\n");
        }
        
        if most_recent_ema50 > most_recent_ema200 {
            result.push_str("Long-term EMA Trend: Bullish (50 above 200)\n");
        } else {
            result.push_str("Long-term EMA Trend: Bearish (50 below 200)\n");
        }
        
        // Check for potential golden/death cross forming
        if most_recent_ema50 < most_recent_ema200 && most_recent_ema50 / most_recent_ema200 > 0.995 {
            result.push_str("Alert: Potential golden cross forming (50 EMA approaching 200 EMA from below)\n");
        } else if most_recent_ema50 > most_recent_ema200 && most_recent_ema50 / most_recent_ema200 < 1.005 {
            result.push_str("Alert: Potential death cross forming (50 EMA approaching 200 EMA from above)\n");
        }
    } else if price_values.len() >= 20 {
        let mut ema12 = ExponentialMovingAverage::new(12).unwrap();
        let mut ema26 = ExponentialMovingAverage::new(26).unwrap();
        
        // Store last 5 values for each indicator
        let mut ema12_values = Vec::new();
        let mut ema26_values = Vec::new();
        let mut timestamps = Vec::new();
        
        // Get timestamps for the last 5 periods
        if data.prices.len() >= 5 {
            for i in (data.prices.len() - 5)..data.prices.len() {
                timestamps.push(data.prices[i].0);
            }
        }
        
        // Process all prices to calculate indicators
        for (i, &price) in price_values.iter().enumerate() {
            let ema12_val = ema12.next(price);
            let ema26_val = ema26.next(price);
            
            // Only keep the last 5 values
            if i >= price_values.len() - 5 {
                ema12_values.push(ema12_val);
                ema26_values.push(ema26_val);
            }
        }
        
        result.push_str("\nExponential Moving Averages (Last 5 periods):\n");
        
        // Display timestamps and EMA values for the last 5 periods
        for i in 0..min(5, ema12_values.len()) {
            let date = if i < timestamps.len() {
                DateTime::<Utc>::from_timestamp((timestamps[i] as i64) / 1000, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                format!("Period -{}", 5-i)
            };
            
            result.push_str(&format!("{}:\n", date));
            result.push_str(&format!("  EMA (12-day): ${:.2}\n", ema12_values[i]));
            result.push_str(&format!("  EMA (26-day): ${:.2}\n", ema26_values[i]));
        }
        
        // Add trend indication based on EMA crossover
        let most_recent_ema12 = *ema12_values.last().unwrap();
        let most_recent_ema26 = *ema26_values.last().unwrap();
        
        result.push_str("\nEMA Trend Analysis:\n");
        if most_recent_ema12 > most_recent_ema26 {
            result.push_str("Trend: Bullish (Short-term EMA above Long-term EMA)\n");
        } else {
            result.push_str("Trend: Bearish (Short-term EMA below Long-term EMA)\n");
        }
    }
      // Calculate RSI (Relative Strength Index)
    if price_values.len() >= 14 {
        let mut rsi = RelativeStrengthIndex::new(14).unwrap();
        
        // Store last 5 RSI values
        let mut rsi_values = Vec::new();
        let mut timestamps = Vec::new();
        
        // Get timestamps for the last 5 periods
        if data.prices.len() >= 5 {
            for i in (data.prices.len() - 5)..data.prices.len() {
                timestamps.push(data.prices[i].0);
            }
        }
        
        // Process all prices to calculate RSI
        for (i, &price) in price_values.iter().enumerate() {
            let rsi_val = rsi.next(price);
            
            // Only keep the last 5 values
            if i >= price_values.len() - 5 {
                rsi_values.push(rsi_val);
            }
        }
        
        result.push_str("\nRSI With EMA (14-day) - Last 5 periods:\n");
        
        // Display timestamps and RSI values for the last 5 periods
        for i in 0..min(5, rsi_values.len()) {
            let date = if i < timestamps.len() {
                DateTime::<Utc>::from_timestamp((timestamps[i] as i64) / 1000, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                format!("Period -{}", 5-i)
            };
            
            let rsi_val = rsi_values[i];
            let rsi_interpretation = if rsi_val > 70.0 {
                "Overbought (>70)"
            } else if rsi_val < 30.0 {
                "Oversold (<30)"
            } else {
                "Neutral (30-70)"
            };
            
            result.push_str(&format!("{}: {:.2} - {}\n", date, rsi_val, rsi_interpretation));
        }
        
        // Add RSI trend analysis
        if rsi_values.len() >= 2 {
            let last_rsi = *rsi_values.last().unwrap();
            let prev_rsi = rsi_values[rsi_values.len() - 2];
            
            result.push_str("\nRSI Trend: ");
            if last_rsi > prev_rsi {
                result.push_str("Rising (Increasing momentum)\n");
            } else if last_rsi < prev_rsi {
                result.push_str("Falling (Decreasing momentum)\n");
            } else {
                result.push_str("Flat (Stable momentum)\n");
            }
        }
    }
      // Calculate MACD (12, 26, 9)
    if price_values.len() >= 35 { // Need at least 26 + 9 data points
        let mut macd = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
        
        // Store last 5 MACD values
        let mut macd_values = Vec::new();
        let mut timestamps = Vec::new();
        
        // Get timestamps for the last 5 periods
        if data.prices.len() >= 5 {
            for i in (data.prices.len() - 5)..data.prices.len() {
                timestamps.push(data.prices[i].0);
            }
        }
        
        // Process all prices to calculate MACD
        for (i, &price) in price_values.iter().enumerate() {
            let macd_val = macd.next(price);
            
            // Only keep the last 5 values
            if i >= price_values.len() - 5 {
                macd_values.push(macd_val);
            }
        }
        
        result.push_str("\nMACD (12, 26, 9) - Last 5 periods:\n");
        
        // Display timestamps and MACD values for the last 5 periods
        for i in 0..min(5, macd_values.len()) {            let date = if i < timestamps.len() {
                DateTime::<Utc>::from_timestamp((timestamps[i] as i64) / 1000, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                format!("Period -{}", 5-i)
            };
            
            let macd_val = &macd_values[i];
            
            result.push_str(&format!("{}:\n", date));
            result.push_str(&format!("  MACD Line: {:.2}\n", macd_val.macd));
            result.push_str(&format!("  Signal Line: {:.2}\n", macd_val.signal));
            result.push_str(&format!("  Histogram: {:.2}\n", macd_val.histogram));
              // Add per-period MACD interpretation
            let interpretation = if macd_val.macd > macd_val.signal {
                if macd_val.macd > 0.0 && macd_val.signal > 0.0 {
                    "Bullish (MACD above Signal, both positive)"
                } else {
                    "Potential bullish crossover (below zero)"
                }
            } else {
                if macd_val.macd < 0.0 && macd_val.signal < 0.0 {
                    "Bearish (MACD below Signal, both negative)"
                } else {
                    "Potential bearish crossover (above zero)"
                }
            };
            
            result.push_str(&format!("  Indication: {}\n", interpretation));
        }
          // Add MACD trend analysis based on most recent values
        if macd_values.len() >= 2 {
            let last_macd = &macd_values.last().unwrap();
            let prev_macd = &macd_values[macd_values.len() - 2];
            
            result.push_str("\nMACD Trend Analysis:\n");
            
            // Check if a crossover occurred in the last period
            let current_above_signal = last_macd.macd > last_macd.signal;
            let prev_above_signal = prev_macd.macd > prev_macd.signal;
            
            if current_above_signal && !prev_above_signal {
                result.push_str("Signal: Bullish crossover (MACD crossed above Signal)\n");
            } else if !current_above_signal && prev_above_signal {
                result.push_str("Signal: Bearish crossover (MACD crossed below Signal)\n");
            } else if current_above_signal {
                result.push_str("Signal: Bullish momentum continues\n");
            } else {
                result.push_str("Signal: Bearish momentum continues\n");
            }
              // Check histogram direction for momentum
            if last_macd.histogram > prev_macd.histogram {
                result.push_str("Momentum: Increasing (Histogram rising)\n");
            } else {
                result.push_str("Momentum: Decreasing (Histogram falling)\n");
            }
        }
    }
      // Bollinger Bands (20, 2)
    if price_values.len() >= 20 {
        let mut bb = BollingerBands::new(20, 2.0).unwrap();
        
        // Store last 5 Bollinger Bands values
        let mut bb_values = Vec::new();
        let mut timestamps = Vec::new();
        
        // Get timestamps for the last 5 periods
        if data.prices.len() >= 5 {
            for i in (data.prices.len() - 5)..data.prices.len() {
                timestamps.push(data.prices[i].0);
            }
        }
        
        // Process all prices to calculate Bollinger Bands
        for (i, &price) in price_values.iter().enumerate() {
            let bb_val = bb.next(price);
            
            // Only keep the last 5 values
            if i >= price_values.len() - 5 {
                bb_values.push((bb_val, price));
            }
        }
        
        result.push_str("\nBollinger Bands (20, 2) - Last 5 periods:\n");
        
        // Display timestamps and Bollinger Bands values for the last 5 periods
        for i in 0..min(5, bb_values.len()) {
            let date = if i < timestamps.len() {
                DateTime::<Utc>::from_timestamp((timestamps[i] as i64) / 1000, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            } else {
                format!("Period -{}", 5-i)
            };
            
            let (bb_val, price) = &bb_values[i];
            
            // Calculate price position within bands
            let band_width = bb_val.upper - bb_val.lower;
            let position = (price - bb_val.lower) / band_width * 100.0;
            
            // Determine the interpretation
            let interpretation = if *price > bb_val.upper {
                "Potentially overbought (price above upper band)"
            } else if *price < bb_val.lower {
                "Potentially oversold (price below lower band)"
            } else {
                "Within normal trading range"
            };
            
            result.push_str(&format!("{}:\n", date));
            result.push_str(&format!("  Upper Band: ${:.2}\n", bb_val.upper));
            result.push_str(&format!("  Middle Band (SMA): ${:.2}\n", bb_val.average));
            result.push_str(&format!("  Lower Band: ${:.2}\n", bb_val.lower));
            result.push_str(&format!("  Price: ${:.2}\n", price));
            result.push_str(&format!("  Position: {:.1}% of band width from lower band\n", position));
            result.push_str(&format!("  Indication: {}\n", interpretation));
        }
        
        // Add Bollinger Bands width analysis for volatility assessment
        if bb_values.len() >= 2 {
            let (last_bb, _) = bb_values.last().unwrap();
            let (prev_bb, _) = &bb_values[bb_values.len() - 2];
            
            let last_width = last_bb.upper - last_bb.lower;
            let prev_width = prev_bb.upper - prev_bb.lower;
            
            result.push_str("\nBollinger Bands Volatility Analysis:\n");
            if last_width > prev_width {
                result.push_str("Volatility: Increasing (Bands widening)\n");
            } else if last_width < prev_width {
                result.push_str("Volatility: Decreasing (Bands narrowing)\n");
            } else {
                result.push_str("Volatility: Stable (Band width unchanged)\n");
            }
        }
    }
      // On Balance Volume (OBV)
    if !price_values.is_empty() && !volume_values.is_empty() && price_values.len() == volume_values.len() {
        // For OBV, we'll calculate it manually since the ta library implementation is causing issues
        let mut obv_value = 0.0;
        let mut obv_values = vec![obv_value];
        
        // Start from index 1 to have a previous price to compare
        let mut prev_price = price_values[0];
        for i in 1..price_values.len() {
            // Calculate OBV manually according to the formula
            let current_price = price_values[i];
            let current_volume = volume_values[i];
            
            if current_price > prev_price {
                obv_value += current_volume;  // Price up, add volume
            } else if current_price < prev_price {
                obv_value -= current_volume;  // Price down, subtract volume
            } // If price unchanged, obv remains the same
            
            obv_values.push(obv_value);
            prev_price = current_price;
        }
        
        if obv_values.len() >= 5 {
            result.push_str("\nOn Balance Volume (OBV) - Last 5 periods:\n");
            
            // Get timestamps for the last 5 periods
            let mut timestamps = Vec::new();
            if data.prices.len() >= 5 {
                for i in (data.prices.len() - 5)..data.prices.len() {
                    timestamps.push(data.prices[i].0);
                }
            }
            
            // Display the last 5 OBV values
            for i in 0..5 {
                let idx = obv_values.len() - 5 + i;
                let current_obv = obv_values[idx];
                let prev_obv = if idx > 0 { obv_values[idx - 1] } else { 0.0 };
                let obv_change = ((current_obv - prev_obv) / current_obv.abs().max(1.0)) * 100.0;
                
                let date = if i < timestamps.len() {
                    DateTime::<Utc>::from_timestamp((timestamps[i] as i64) / 1000, 0)
                        .unwrap()
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                } else {
                    format!("Period -{}", 5-i)
                };
                
                let interpretation = if obv_change > 2.0 {
                    "Strong buying pressure (OBV increasing)"
                } else if obv_change < -2.0 {
                    "Strong selling pressure (OBV decreasing)"
                } else {
                    "Neutral volume pressure"
                };
                
                result.push_str(&format!("{}:\n", date));
                result.push_str(&format!("  OBV: {:.0}\n", current_obv));
                result.push_str(&format!("  Change: {:.2}%\n", obv_change));
                result.push_str(&format!("  Indication: {}\n", interpretation));
            }
            
            // Add OBV trend analysis
            let last_obv = *obv_values.last().unwrap();
            let obv_5_period_ago = obv_values[obv_values.len() - 5];
            let overall_change = (last_obv - obv_5_period_ago) / last_obv.abs().max(1.0) * 100.0;
            
            result.push_str("\nOBV 5-Period Trend Analysis:\n");
            if overall_change > 5.0 {
                result.push_str("Strong buying pressure over last 5 periods (OBV trending up)\n");
            } else if overall_change < -5.0 {
                result.push_str("Strong selling pressure over last 5 periods (OBV trending down)\n");
            } else {
                result.push_str("Neutral volume pressure over last 5 periods\n");
            }
        }
    }
      // Average True Range (ATR)
    if high_values.len() >= 14 && low_values.len() >= 14 && price_values.len() >= 14 {
        let mut atr = AverageTrueRange::new(14).unwrap();
        
        // Store ATR values
        let mut atr_values = Vec::new();
        let mut timestamps = Vec::new();
        
        // Get timestamps for the last 5 periods
        if data.prices.len() >= 5 {
            for i in (data.prices.len() - 5)..data.prices.len() {
                timestamps.push(data.prices[i].0);
            }
        }
        
        // Process all prices to calculate ATR
        for i in 1..price_values.len() {
            if i < high_values.len() && i < low_values.len() {
                // Calculate true range manually
                let high = high_values[i];
                let low = low_values[i];
                let prev_close = price_values[i-1];
                
                // True Range is the greatest of:
                // 1. Current High - Current Low
                // 2. |Current High - Previous Close|
                // 3. |Current Low - Previous Close|
                let range1 = high - low;
                let range2 = (high - prev_close).abs();
                let range3 = (low - prev_close).abs();
                
                let true_range = range1.max(range2).max(range3);
                let atr_val = atr.next(true_range);
                
                // Only store the last 5 ATR values
                if i >= price_values.len() - 5 {
                    atr_values.push((atr_val, price_values[i]));
                }
            }
        }
        
        if atr_values.len() >= 1 {
            result.push_str("\nAverage True Range (ATR) - Last 5 periods:\n");
            
            // Display timestamps and ATR values for the last 5 periods
            for i in 0..min(5, atr_values.len()) {
                let date = if i < timestamps.len() {
                    DateTime::<Utc>::from_timestamp((timestamps[i] as i64) / 1000, 0)
                        .unwrap()
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                } else {
                    format!("Period -{}", 5-i)
                };
                
                let (atr_val, price) = atr_values[i];
                let atr_percent = atr_val / price * 100.0;
                
                let volatility = if atr_percent > 5.0 {
                    "High (ATR > 5% of price)"
                } else if atr_percent > 3.0 {
                    "Medium (ATR 3-5% of price)"
                } else {
                    "Low (ATR < 3% of price)"
                };
                
                result.push_str(&format!("{}:\n", date));
                result.push_str(&format!("  ATR (14-day): ${:.2}\n", atr_val));
                result.push_str(&format!("  ATR as % of price: {:.2}%\n", atr_percent));
                result.push_str(&format!("  Volatility: {}\n", volatility));
            }
            
            // Add ATR trend analysis if we have enough data
            if atr_values.len() >= 2 {
                let (last_atr, _) = *atr_values.last().unwrap();
                let (prev_atr, _) = atr_values[atr_values.len() - 2];
                
                result.push_str("\nATR Trend Analysis:\n");
                if last_atr > prev_atr {
                    result.push_str("Volatility: Increasing (ATR rising)\n");
                } else if last_atr < prev_atr {
                    result.push_str("Volatility: Decreasing (ATR falling)\n");
                } else {
                    result.push_str("Volatility: Stable (ATR unchanged)\n");
                }
            }
        }
    }
    
    // Support and resistance levels (simple implementation)
    let (support, resistance) = calculate_support_resistance(&price_values);
    result.push_str(&format!("\nSupport level: ${:.2}\n", support));
    result.push_str(&format!("Resistance level: ${:.2}\n", resistance));
    
    result
}

/// Calculate simple support and resistance levels
fn calculate_support_resistance(prices: &[f64]) -> (f64, f64) {
    if prices.is_empty() {
        return (0.0, 0.0);
    }
    
    let min_price = *prices.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);
    let max_price = *prices.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);
    
    // Simple implementation - using recent min/max as support/resistance
    (min_price, max_price)
}