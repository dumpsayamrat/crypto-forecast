use chrono::{DateTime, Utc};
use crate::data_fetcher::{CryptoData, FearGreedData};
use ta::indicators::{
    MovingAverageConvergenceDivergence, RelativeStrengthIndex,
    ExponentialMovingAverage, SimpleMovingAverage, 
    BollingerBands, AverageTrueRange
};
use ta::Next;

/// Format Bitcoin data into a string for analysis, including technical indicators
pub fn format_data_for_analysis(data: &CryptoData, fng: &Vec<FearGreedData>) -> String {
    let mut formatted_data = String::new();
    
    // Check if OHLC data is available and non-empty
    if !data.ohlc_data.is_empty() {
        formatted_data.push_str("Bitcoin historical OHLC + Volume data from Binance:\n");
        formatted_data.push_str("Date,Open,High,Low,Close,Volume\n");
        
        // Get the full dataset
        for i in 0..data.ohlc_data.len() {
            let (timestamp, _, _, _, close, volume) = data.ohlc_data[i];
            let date = DateTime::<Utc>::from_timestamp((timestamp as i64) / 1000, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            
            formatted_data.push_str(&format!("{}: C=${:.2} V={:.2}\n", 
                date, close, volume));
        }
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
                
            formatted_data.push_str(&format!("{}: ${:.2}\n", date, price));
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
        
        // Process all prices in chronological order to get final values
        let mut sma7_value = 0.0;
        let mut sma20_value = 0.0;
        let mut sma50_value = 0.0;
        let mut sma200_value = 0.0;
        for &price in &price_values {
            sma7_value = sma7.next(price);
            sma20_value = sma20.next(price);
            sma50_value = sma50.next(price);
            sma200_value = sma200.next(price);
        }
        
        result.push_str("\nSimple Moving Averages:\n");
        result.push_str(&format!("SMA (7-period): ${:.2}\n", sma7_value));
        result.push_str(&format!("SMA (20-period): ${:.2}\n", sma20_value));
        result.push_str(&format!("SMA (50-period): ${:.2}\n", sma50_value));
        result.push_str(&format!("SMA (200-period): ${:.2}\n", sma200_value));
        
        // Add trend indications based on SMA crossovers
        if sma7_value > sma20_value {
            result.push_str("Short-term Trend: Bullish (7 above 20)\n");
        } else {
            result.push_str("Short-term Trend: Bearish (7 below 20)\n");
        }
        
        if sma50_value > sma200_value {
            result.push_str("Long-term Trend: Bullish (50 above 200, Golden Cross active)\n");
        } else {
            result.push_str("Long-term Trend: Bearish (50 below 200, Death Cross active)\n");
        }
        
        // Price position relative to major SMAs
        let current_price = *price_values.last().unwrap();
        result.push_str(&format!("Price relative to SMAs: {}\n", 
            if current_price > sma200_value && current_price > sma50_value {
                "Strong bullish (Price above both 50 & 200 SMAs)"
            } else if current_price > sma200_value {
                "Moderately bullish (Price above 200 SMA but below 50 SMA)"
            } else if current_price > sma50_value {
                "Mixed signals (Price above 50 SMA but below 200 SMA)"
            } else {
                "Bearish (Price below both 50 & 200 SMAs)"
            }
        ));
    } else if price_values.len() >= 20 {
        let mut sma7 = SimpleMovingAverage::new(7).unwrap();
        let mut sma20 = SimpleMovingAverage::new(20).unwrap();
        
        // Process all prices in chronological order to get final values
        let mut sma7_value = 0.0;
        let mut sma20_value = 0.0;
        for &price in &price_values {
            sma7_value = sma7.next(price);
            sma20_value = sma20.next(price);
        }
        
        result.push_str("\nSimple Moving Averages:\n");
        result.push_str(&format!("SMA (7-day): ${:.2}\n", sma7_value));
        result.push_str(&format!("SMA (20-day): ${:.2}\n", sma20_value));
        
        // Add trend indication based on SMA crossover
        if sma7_value > sma20_value {
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
        
        // Process all prices in chronological order
        let mut ema12_value = 0.0;
        let mut ema26_value = 0.0;
        let mut ema50_value = 0.0;
        let mut ema200_value = 0.0;
        for &price in &price_values {
            ema12_value = ema12.next(price);
            ema26_value = ema26.next(price);
            ema50_value = ema50.next(price);
            ema200_value = ema200.next(price);
        }
        
        result.push_str("\nExponential Moving Averages:\n");
        result.push_str(&format!("EMA (12-period): ${:.2}\n", ema12_value));
        result.push_str(&format!("EMA (26-period): ${:.2}\n", ema26_value));
        result.push_str(&format!("EMA (50-period): ${:.2}\n", ema50_value));
        result.push_str(&format!("EMA (200-period): ${:.2}\n", ema200_value));
        
        // Add trend indications based on EMA crossovers
        if ema12_value > ema26_value {
            result.push_str("Short-term EMA Trend: Bullish (12 above 26)\n");
        } else {
            result.push_str("Short-term EMA Trend: Bearish (12 below 26)\n");
        }
        
        if ema50_value > ema200_value {
            result.push_str("Long-term EMA Trend: Bullish (50 above 200)\n");
        } else {
            result.push_str("Long-term EMA Trend: Bearish (50 below 200)\n");
        }
        
        // Check for potential golden/death cross forming
        if ema50_value < ema200_value && ema50_value / ema200_value > 0.995 {
            result.push_str("Alert: Potential golden cross forming (50 EMA approaching 200 EMA from below)\n");
        } else if ema50_value > ema200_value && ema50_value / ema200_value < 1.005 {
            result.push_str("Alert: Potential death cross forming (50 EMA approaching 200 EMA from above)\n");
        }
    } else if price_values.len() >= 20 {
        let mut ema12 = ExponentialMovingAverage::new(12).unwrap();
        let mut ema26 = ExponentialMovingAverage::new(26).unwrap();
        
        // Process all prices in chronological order
        let mut ema12_value = 0.0;
        let mut ema26_value = 0.0;
        for &price in &price_values {
            ema12_value = ema12.next(price);
            ema26_value = ema26.next(price);
        }
        
        result.push_str("\nExponential Moving Averages:\n");
        result.push_str(&format!("EMA (12-day): ${:.2}\n", ema12_value));
        result.push_str(&format!("EMA (26-day): ${:.2}\n", ema26_value));
        
        // Add trend indication based on EMA crossover
        if ema12_value > ema26_value {
            result.push_str("Trend: Bullish (Short-term EMA above Long-term EMA)\n");
        } else {
            result.push_str("Trend: Bearish (Short-term EMA below Long-term EMA)\n");
        }
    }
    
    // Calculate RSI (Relative Strength Index)
    if price_values.len() >= 14 {
        let mut rsi = RelativeStrengthIndex::new(14).unwrap();
        
        // Process all prices in chronological order
        let mut rsi_value = 0.0;
        for &price in &price_values {
            rsi_value = rsi.next(price);
        }
        
        result.push_str(&format!("\nRSI With EMA (14-day): {:.2}\n", rsi_value));
        
        // Add RSI interpretation
        if rsi_value > 70.0 {
            result.push_str("RSI Indication: Overbought (>70)\n");
        } else if rsi_value < 30.0 {
            result.push_str("RSI Indication: Oversold (<30)\n");
        } else {
            result.push_str("RSI Indication: Neutral (30-70)\n");
        }
    }
    
    // Calculate MACD (12, 26, 9)
    if price_values.len() >= 35 { // Need at least 26 + 9 data points
        let mut macd = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
        
        // Process all prices in chronological order
        let mut macd_value = None;
        for &price in &price_values {
            macd_value = Some(macd.next(price));
        }
        
        // Get the most recent MACD value
        let last_macd = macd_value.unwrap();
        
        result.push_str("\nMACD (12, 26, 9):\n");
        result.push_str(&format!("MACD Line: {:.2}\n", last_macd.macd));
        result.push_str(&format!("Signal Line: {:.2}\n", last_macd.signal));
        result.push_str(&format!("Histogram: {:.2}\n", last_macd.histogram));
        
        // Add MACD interpretation
        if last_macd.macd > last_macd.signal {
            result.push_str("MACD Indication: Bullish (MACD Line above Signal Line)\n");
            
            if last_macd.macd > 0.0 && last_macd.signal > 0.0 {
                result.push_str("MACD Strength: Strong bullish momentum (both lines above zero)\n");
            } else {
                result.push_str("MACD Strength: Potential bullish crossover (below zero)\n");
            }
        } else {
            result.push_str("MACD Indication: Bearish (MACD Line below Signal Line)\n");
            
            if last_macd.macd < 0.0 && last_macd.signal < 0.0 {
                result.push_str("MACD Strength: Strong bearish momentum (both lines below zero)\n");
            } else {
                result.push_str("MACD Strength: Potential bearish crossover (above zero)\n");
            }
        }
    }
    
    // Bollinger Bands (20, 2)
    if price_values.len() >= 20 {
        let mut bb = BollingerBands::new(20, 2.0).unwrap();
        
        // Process all prices in chronological order
        let mut bb_values = vec![];
        for &price in &price_values {
            bb_values.push(bb.next(price));
        }
        
        // Get the most recent BB values
        let last_bb = bb_values.last().unwrap();
        let current_price = *price_values.last().unwrap();
        
        result.push_str("\nBollinger Bands (20, 2):\n");
        result.push_str(&format!("Upper Band: ${:.2}\n", last_bb.upper));
        result.push_str(&format!("Middle Band (SMA): ${:.2}\n", last_bb.average));
        result.push_str(&format!("Lower Band: ${:.2}\n", last_bb.lower));
        
        // Add Bollinger Bands interpretation
        let band_width = last_bb.upper - last_bb.lower;
        let position = (current_price - last_bb.lower) / band_width * 100.0;
        
        result.push_str(&format!("Price Position: {:.1}% of band width from lower band\n", position));
        
        if current_price > last_bb.upper {
            result.push_str("BB Indication: Potentially overbought (price above upper band)\n");
        } else if current_price < last_bb.lower {
            result.push_str("BB Indication: Potentially oversold (price below lower band)\n");
        } else {
            result.push_str("BB Indication: Within normal trading range\n");
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
        
        if !obv_values.is_empty() {
            let last_obv = *obv_values.last().unwrap();
            let obv_change = if obv_values.len() > 5 {
                (last_obv - obv_values[obv_values.len() - 5]) / last_obv.abs() * 100.0
            } else {
                0.0
            };
            
            result.push_str("\nOn Balance Volume (OBV):\n");
            result.push_str(&format!("Current OBV: {:.0}\n", last_obv));
            result.push_str(&format!("5-period OBV Change: {:.2}%\n", obv_change));
            
            // Add OBV interpretation
            if obv_change > 5.0 {
                result.push_str("OBV Indication: Strong buying pressure (OBV increasing)\n");
            } else if obv_change < -5.0 {
                result.push_str("OBV Indication: Strong selling pressure (OBV decreasing)\n");
            } else {
                result.push_str("OBV Indication: Neutral volume pressure\n");
            }
        }
    }
    
    // Average True Range (ATR)
    if high_values.len() >= 14 && low_values.len() >= 14 && price_values.len() >= 14 {
        let mut atr = AverageTrueRange::new(14).unwrap();
        
        // Process all prices to get ATR values
        let mut atr_values = vec![];
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
                atr_values.push(atr.next(true_range));
            }
        }
        
        if !atr_values.is_empty() {
            let last_atr = *atr_values.last().unwrap();
            let current_price = *price_values.last().unwrap();
            let atr_percent = last_atr / current_price * 100.0;
            
            result.push_str("\nAverage True Range (ATR):\n");
            result.push_str(&format!("14-day ATR: ${:.2}\n", last_atr));
            result.push_str(&format!("ATR as % of price: {:.2}%\n", atr_percent));
            
            // Add ATR interpretation
            if atr_percent > 5.0 {
                result.push_str("Volatility: High (ATR > 5% of price)\n");
            } else if atr_percent > 3.0 {
                result.push_str("Volatility: Medium (ATR 3-5% of price)\n");
            } else {
                result.push_str("Volatility: Low (ATR < 3% of price)\n");
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