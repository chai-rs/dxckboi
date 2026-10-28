//! Position sizing: risk a fixed fraction of equity per trade.

/// Order size constraints from the instrument and from our own risk limits.
#[derive(Clone, Copy, Debug)]
pub struct SizeLimits {
    /// Sizes must be a multiple of this (the instrument's size increment).
    pub increment: f64,
    /// Smallest order the venue accepts (0.0 if unknown).
    pub min_units: f64,
    /// Largest order the venue accepts (f64::INFINITY if unknown).
    pub max_units: f64,
    /// Our own cap on entry notional, in quote currency.
    pub max_notional: f64,
}

/// Units to trade so that hitting the stop loses `risk_fraction` of `equity`,
/// within `limits`. Returns 0.0 when no valid size exists.
pub fn units_for_risk(
    equity: f64,
    risk_fraction: f64,
    entry: f64,
    stop: f64,
    limits: SizeLimits,
) -> f64 {
    let stop_distance = (entry - stop).abs();
    if equity <= 0.0 || stop_distance <= 0.0 || entry <= 0.0 || limits.increment <= 0.0 {
        return 0.0;
    }
    let by_risk = equity * risk_fraction / stop_distance;
    let units = by_risk
        .min(limits.max_notional / entry)
        .min(limits.max_units);
    // f64 subtraction leaves tiny errors (0.7000 - 0.6950 = 0.0050000000000000044),
    // so nudge by an epsilon before flooring or exact sizes lose one increment.
    let units = (units / limits.increment + 1e-9).floor() * limits.increment;
    if units < limits.min_units { 0.0 } else { units }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPEN: SizeLimits = SizeLimits {
        increment: 1.0,
        min_units: 0.0,
        max_units: f64::INFINITY,
        max_notional: 1e9,
    };

    #[test]
    fn risks_the_requested_fraction() {
        // 1% of 100k = 1,000 USD over a 0.0050 stop distance = 200,000 units.
        assert_eq!(
            units_for_risk(100_000.0, 0.01, 0.7000, 0.6950, OPEN),
            200_000.0
        );
    }

    #[test]
    fn notional_cap_wins_when_smaller() {
        let limits = SizeLimits {
            max_notional: 70_000.0,
            ..OPEN
        };
        assert_eq!(
            units_for_risk(100_000.0, 0.01, 0.7000, 0.6950, limits),
            100_000.0
        );
    }

    #[test]
    fn venue_maximum_is_respected() {
        let limits = SizeLimits {
            max_units: 150_000.0,
            ..OPEN
        };
        assert_eq!(
            units_for_risk(100_000.0, 0.01, 0.7000, 0.6950, limits),
            150_000.0
        );
    }

    #[test]
    fn rounds_down_to_the_size_increment() {
        let limits = SizeLimits {
            increment: 1_000.0,
            ..OPEN
        };
        assert_eq!(
            units_for_risk(100_000.0, 0.01, 0.7000, 0.6930, limits),
            142_000.0
        );
    }

    #[test]
    fn below_venue_minimum_is_not_traded() {
        let limits = SizeLimits {
            min_units: 500_000.0,
            ..OPEN
        };
        assert_eq!(units_for_risk(100_000.0, 0.01, 0.7000, 0.6950, limits), 0.0);
    }

    #[test]
    fn zero_stop_distance_is_not_sized() {
        assert_eq!(units_for_risk(100_000.0, 0.01, 0.7, 0.7, OPEN), 0.0);
    }
}
