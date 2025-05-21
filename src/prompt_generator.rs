/// Generate a trading recommendation prompt
pub fn generate_trading_recommendation_prompt(data: &str) -> String {
    format!(
        "You are a cryptocurrency market analyst specializing in Bitcoin. Your task is to provide an insightful summary of the Bitcoin market, including price predictions, buy and sell positions, key levels, risk assessment, and overall recommendations. Use the following data to conduct your analysis:\n\
        \n\
        <historical_data>\n\
        {}\n\
        </historical_data>\n\
        \n\
        Analyze the provided data carefully, paying attention to trends, patterns, and signals from various indicators. Consider both technical and sentiment factors in your analysis.\n\
        \n\
        Prepare a comprehensive summary report with the following sections:\n\
        \n\
        1. Market Overview: Provide a brief overview of the current Bitcoin market situation based on the latest data points.\n\
        \n\
        2. Price Prediction: Offer price predictions for short-term (1-7 days), mid-term (1-3 months), and long-term (6-12 months) horizons. Support your predictions with relevant data and indicator analysis.\n\
        \n\
        3. Buy and Sell Positions: Recommend entry and exit points for short, mid, and long-term traders. Explain the rationale behind each position.\n\
        \n\
        4. Key Levels: Identify and explain important support and resistance levels to watch. Provide specific price points and reasons why these levels are significant.\n\
        \n\
        5. Indicator Analysis: Analyze each of the following indicators and explain their implications for Bitcoin's price action:\n\
           - RSI with EMA (overbought/oversold conditions)\n\
           - MACD (trend strength and momentum)\n\
           - Bollinger Bands (volatility and potential reversals)\n\
           - SMA and EMA crossovers (trend direction)\n\
           - OBV (volume confirmation of trends)\n\
           - ATR (volatility measurement)\n\
           - Fear and Greed Index (market sentiment)\n\
        \n\
        6. Risk Assessment: Evaluate the overall risk level (low, medium, or high) for Bitcoin investments at this time. Provide a detailed explanation for your assessment, considering both technical and fundamental factors.\n\
        \n\
        7. Timeframe Recommendations: Offer specific recommendations for short-term, medium-term, and long-term investors. Explain how your advice differs for each timeframe and why.\n\
        \n\
        8. Overall Recommendation: Conclude with an overall recommendation to Buy, Sell, or Hold Bitcoin. Justify your recommendation based on the analysis of all indicators and market factors discussed in the report.\n\
        \n\
        Before providing your final output, use <scratchpad> tags to organize your thoughts and analyze the data. This will help you formulate a well-reasoned and comprehensive report.\n\
        \n\
        Present your final analysis and recommendations within <bitcoin_market_analysis> tags. Ensure that your report is well-structured, easy to read, and provides clear, actionable insights for investors with different time horizons.", 
        data
    )
}
