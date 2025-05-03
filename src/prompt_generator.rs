/// Generate a trading recommendation prompt
pub fn generate_trading_recommendation_prompt(data: &str) -> String {
    format!(
        "You are a cryptocurrency trading expert. Based on the following Bitcoin price data and technical indicators, please provide:\n\
        \n\
        1. Current market analysis (bullish, bearish, or neutral outlook) based on these technical indicators and fear and greed indexes:\n\
           - RSI with EMA (overbought/oversold conditions)\n\
           - MACD (trend strength and momentum)\n\
           - Bollinger Bands (volatility and potential reversals)\n\
           - SMA and EMA crossovers (trend direction)\n\
           - OBV (volume confirmation of trends)\n\
           - ATR (volatility measurement)\n\
           - Fear and Greed Index (market sentiment)\n\
        2. Specific buy positions with:\n\
           - Entry price levels\n\
           - Stop loss levels\n\
           - Take profit targets\n\
        3. Specific sell positions with:\n\
           - Entry price levels\n\
           - Stop loss levels\n\
           - Take profit targets\n\
        4. Key support and resistance levels to watch\n\
        5. Risk assessment (low, medium, high) with explanation\n\
        6. Timeframe for these recommendations (short-term, medium-term, long-term)\n\
        7. Predict Bitcoin’s price range 24 hours, next 7 days and next 30 days from now (min, max, likely value)\n\
        8. Overall Recommend: Buy, Sell, or Hold.\n\
        9. Explain your reasoning in terms of the indicators.\n\
        \n\
        Present your analysis in a clear, structured format. Include specific price levels rather than percentages when possible.\n\
        \n\
        DATA:\n{}", 
        data
    )
}
