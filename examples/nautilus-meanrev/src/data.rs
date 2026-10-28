//! Deterministic synthetic bars for backtests and tests.
//!
//! Prices follow a mean-reverting (Ornstein-Uhlenbeck) walk around an anchor
//! that itself drifts as a random walk. The drift creates trending stretches,
//! so a mean-reversion strategy both wins and loses. The data is good for
//! checking that code works; it says nothing about real markets.

use nautilus_core::UnixNanos;
use nautilus_model::{
    data::{Bar, BarType, Data},
    types::{Price, Quantity},
};

/// Small xorshift generator: reproducible across runs and platforms.
struct Rng(u64);

impl Rng {
    fn next_f64(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Approximately standard normal (sum of 12 uniforms).
    fn normal(&mut self) -> f64 {
        (0..12).map(|_| self.next_f64()).sum::<f64>() - 6.0
    }
}

pub struct SyntheticBars {
    pub bar_type: BarType,
    pub start_ns: u64,
    pub interval_ns: u64,
    pub count: usize,
    pub anchor: f64,
    /// Pull toward `anchor` per sub-step (0..1).
    pub reversion: f64,
    /// Standard deviation of each sub-step move.
    pub volatility: f64,
    /// Standard deviation of the anchor's move per bar (0.0 = fixed anchor).
    pub anchor_drift: f64,
    pub price_precision: u8,
    pub seed: u64,
}

impl SyntheticBars {
    pub fn generate(&self) -> Vec<Data> {
        const STEPS_PER_BAR: usize = 10;
        let mut rng = Rng(self.seed.max(1));
        let mut anchor = self.anchor;
        let mut px = anchor;
        let mut bars = Vec::with_capacity(self.count);

        for i in 0..self.count {
            anchor += self.anchor_drift * rng.normal();
            let open = px;
            let (mut high, mut low) = (px, px);
            for _ in 0..STEPS_PER_BAR {
                px += self.reversion * (anchor - px) + self.volatility * rng.normal();
                high = high.max(px);
                low = low.min(px);
            }
            // A bar is stamped at its close.
            let ts = UnixNanos::from(self.start_ns + (i as u64 + 1) * self.interval_ns);
            let p = |v: f64| Price::new(v, self.price_precision);
            bars.push(Data::Bar(Bar::new(
                self.bar_type,
                p(open),
                p(high),
                p(low),
                p(px),
                Quantity::new(1_000_000.0, 0),
                ts,
                ts,
            )));
        }
        bars
    }
}
