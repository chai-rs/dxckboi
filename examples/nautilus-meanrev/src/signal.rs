//! Entry rules, kept free of Nautilus types so they can be unit-tested directly.

/// Band values from the indicators at the close of a bar.
#[derive(Clone, Copy, Debug)]
pub struct Bands {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
    pub atr: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Signal {
    /// Buy, take profit at `take_profit`, stop out at `stop_loss`.
    Long {
        take_profit: f64,
        stop_loss: f64,
    },
    /// Sell, take profit at `take_profit`, stop out at `stop_loss`.
    Short {
        take_profit: f64,
        stop_loss: f64,
    },
    None,
}

/// Fade a close outside the bands back toward the middle band.
///
/// The stop sits `atr_stop` ATRs beyond the close. Trades whose distance to the
/// middle band is under `min_reward_atr` ATRs are skipped: costs would eat them.
pub fn evaluate(close: f64, bands: Bands, atr_stop: f64, min_reward_atr: f64) -> Signal {
    if bands.atr <= 0.0 {
        return Signal::None;
    }
    let reward = (bands.middle - close).abs();
    if reward < min_reward_atr * bands.atr {
        return Signal::None;
    }
    if close < bands.lower {
        Signal::Long {
            take_profit: bands.middle,
            stop_loss: close - atr_stop * bands.atr,
        }
    } else if close > bands.upper {
        Signal::Short {
            take_profit: bands.middle,
            stop_loss: close + atr_stop * bands.atr,
        }
    } else {
        Signal::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BANDS: Bands = Bands {
        upper: 1.0100,
        middle: 1.0000,
        lower: 0.9900,
        atr: 0.0020,
    };

    #[test]
    fn close_below_lower_band_goes_long_toward_middle() {
        let signal = evaluate(0.9890, BANDS, 1.5, 1.0);
        assert_eq!(
            signal,
            Signal::Long {
                take_profit: 1.0000,
                stop_loss: 0.9890 - 0.0030
            }
        );
    }

    #[test]
    fn close_above_upper_band_goes_short() {
        assert!(matches!(
            evaluate(1.0110, BANDS, 1.5, 1.0),
            Signal::Short { .. }
        ));
    }

    #[test]
    fn close_inside_bands_does_nothing() {
        assert_eq!(evaluate(1.0050, BANDS, 1.5, 1.0), Signal::None);
    }

    #[test]
    fn too_little_reward_is_skipped() {
        let narrow = Bands {
            upper: 1.0010,
            middle: 1.0000,
            lower: 0.9990,
            atr: 0.0020,
        };
        assert_eq!(evaluate(0.9985, narrow, 1.5, 1.0), Signal::None);
    }
}
