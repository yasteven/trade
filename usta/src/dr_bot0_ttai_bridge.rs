// trade/usta/src/dr_bot0_ttai_bridge.rs - Bridge layer for dr_bot0 to use either real or mock TTAI

use crate::*;

/// Configuration for TTAI mode
#[derive(Debug, Clone)]
pub enum TtaiMode {
    /// Use real TastyTrade API
    Real,
    
    /// Use mock/simulated TTAI for paper trading
    Mock(dr_ttai_mock::MockTtaiConfig),
}

impl Default for TtaiMode {
    fn default() -> Self {
        TtaiMode::Real
    }
}


/// Helper to create a default mock configuration for testing
pub fn default_test_mock_config() -> dr_ttai_mock::MockTtaiConfig {
    dr_ttai_mock::MockTtaiConfig {
        initial_cash: 100_000.0,
        fill_latency_ms: 10, // Fast fills for testing
        simulate_partial_fills: false,
        partial_fill_probability: 0.0,
        commission_per_contract: 1.0,
        regulatory_fees_per_contract: 0.04,
        auto_generate_chains: true,
        volatility_factor: 0.15,
    }
}

/// Helper to create a realistic mock configuration for paper trading
pub fn default_paper_trading_config() -> dr_ttai_mock::MockTtaiConfig {
    dr_ttai_mock::MockTtaiConfig {
        initial_cash: 25_000.0, // Typical starting capital
        fill_latency_ms: 100, // More realistic latency
        simulate_partial_fills: true,
        partial_fill_probability: 0.15, // 15% chance of partial fill
        commission_per_contract: 1.0,
        regulatory_fees_per_contract: 0.04,
        auto_generate_chains: true,
        volatility_factor: 0.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttai_mode_default() {
        let mode = TtaiMode::default();
        assert!(matches!(mode, TtaiMode::Real));
    }

    #[test]
    fn test_mock_config_creation() {
        let test_config = default_test_mock_config();
        assert_eq!(test_config.initial_cash, 100_000.0);
        assert_eq!(test_config.fill_latency_ms, 10);

        let paper_config = default_paper_trading_config();
        assert_eq!(paper_config.initial_cash, 25_000.0);
        assert!(paper_config.simulate_partial_fills);
    }
}