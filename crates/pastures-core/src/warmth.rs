//! The ranking score. See ADR-0001: warm first, not neglect first.

use crate::config::WarmthConfig;

/// `turns^a / (staleness_hours + floor)^b`. A session with no turns has no warmth.
pub fn warmth(turns: u32, staleness_hours: Option<f64>, cfg: &WarmthConfig) -> f64 {
    let (Some(hours), true) = (staleness_hours, turns > 0) else {
        return 0.0;
    };
    let investment = f64::from(turns).powf(cfg.investment_exponent);
    let staleness = (hours.max(0.0) + cfg.staleness_floor_hours).powf(cfg.staleness_exponent);
    investment / staleness
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> WarmthConfig {
        WarmthConfig::default()
    }

    #[test]
    fn deep_and_recent_beats_shallow_and_recent() {
        assert!(warmth(40, Some(0.1), &cfg()) > warmth(3, Some(0.1), &cfg()));
    }

    #[test]
    fn same_investment_cools_with_time() {
        assert!(warmth(30, Some(0.75), &cfg()) > warmth(30, Some(6.0), &cfg()));
    }

    #[test]
    fn invested_but_cold_sinks_below_shallow_but_hot() {
        // The reversal in ADR-0001: G (20 turns, 3 days) sits under C (3 turns, 10 min).
        assert!(warmth(3, Some(10.0 / 60.0), &cfg()) > warmth(20, Some(72.0), &cfg()));
    }

    #[test]
    fn no_turns_or_no_time_is_zero() {
        assert_eq!(warmth(0, Some(1.0), &cfg()), 0.0);
        assert_eq!(warmth(5, None, &cfg()), 0.0);
    }

    #[test]
    fn floor_keeps_score_finite() {
        assert!(warmth(10, Some(0.0), &cfg()).is_finite());
    }
}
