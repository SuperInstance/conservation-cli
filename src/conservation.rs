//! # Core Conservation Law Mathematics
//!
//! Implements the SuperInstance conservation identity: **γ + η = C**
//! where γ (mutual information), η (conditional entropy), and C (total
//! Shannon entropy) are governed by the chain rule of information theory.
//!
//! ## Key Formula
//!
//! The cancellation delta for a fleet of `n` agents with ternary signals
//! {-−1, 0, +1} is:
//!
//! ```text
//! δ(n) = (1/√n)(1 − 3/(2n))
//! ```
//!
//! As `n → ∞`, δ → 0 and the fleet achieves perfect cancellation.

use rayon::prelude::*;
use std::f64::consts::LN_2;

/// Maximum entropy of a ternary distribution: log₂(3) ≈ 1.585 bits.
pub const LOG2_3: f64 = 1.584_962_500_721_156;

// ─────────────────────────────────────────────────────────────
// Xorshift128+ — lock-free, thread-safe PRNG
// ─────────────────────────────────────────────────────────────

/// Xorshift128+ state. Seeded per-thread for lock-free parallelism.
#[derive(Clone, Copy)]
pub struct Xorshift128Plus {
    s: [u64; 2],
}

impl Xorshift128Plus {
    /// Create a new RNG from a seed.
    pub fn new(seed: u64) -> Self {
        // SplitMix64 to ensure good initial state
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let s0 = {
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        let s1 = {
            z = s0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        Self {
            s: [s0 | 1, s1 | 1], // ensure non-zero
        }
    }

    /// Next 64-bit value.
    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        let x = self.s[0];
        let y = self.s[1];
        self.s[0] = y;
        let x = x ^ (x << 23);
        self.s[1] = x ^ y ^ (x >> 17) ^ (y >> 26);
        self.s[1].wrapping_add(y)
    }

    /// Generate a random ternary signal {−1, 0, +1} with equal probability.
    #[inline(always)]
    pub fn ternary(&mut self) -> i8 {
        let r = self.next_u64();
        // Split the 2^64 range into three as-equal-as-possible buckets.
        // u64::MAX / 3 = 6_148_914_691_236_517_205; the residual 1 value
        // lands in the +1 bucket, giving a bias of < 2^-64.
        const THIRD: u64 = u64::MAX / 3;
        if r < THIRD {
            -1
        } else if r < 2 * THIRD {
            0
        } else {
            1
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Theoretical Predictions
// ─────────────────────────────────────────────────────────────

/// δ(n) = (1/√n)(1 − 3/(2n))
///
/// The theoretical fleet cancellation residual. For small `n` the
/// `3/(2n)` correction dominates; as `n → ∞` the 1/√n tail governs.
pub fn delta(n: usize) -> f64 {
    if n < 2 {
        return 1.0;
    }
    let n = n as f64;
    (1.0 / n.sqrt()) * (1.0 - 3.0 / (2.0 * n))
}

/// Conservation efficiency = 1 − δ(n).
///
/// This is the fraction of signals that cancel in a perfectly random
/// ternary fleet of size `n`.
pub fn efficiency(n: usize) -> f64 {
    1.0 - delta(n)
}

// ─────────────────────────────────────────────────────────────
// Fleet Cancellation
// ─────────────────────────────────────────────────────────────

/// Empirical fleet cancellation: `1 − |Σ signals| / n`.
///
/// Measures how effectively a fleet of ternary signals self-cancels.
/// Returns a value in [0, 1] where 1 = perfect cancellation.
pub fn fleet_cancellation(signals: &[i8]) -> f64 {
    let n = signals.len();
    if n == 0 {
        return 0.0;
    }
    let sum: i64 = signals.iter().map(|&s| s as i64).sum();
    1.0 - (sum.unsigned_abs() as f64 / n as f64)
}

// ─────────────────────────────────────────────────────────────
// Monte Carlo Simulation (Rayon-parallel)
// ─────────────────────────────────────────────────────────────

/// Monte Carlo fleet cancellation over `n_trials` random fleets.
///
/// Each trial generates `n_agents` ternary signals, computes the
/// cancellation factor, and returns the mean. Uses Rayon for
/// embarrassingly parallel execution with per-thread RNG.
pub fn monte_carlo(n_agents: usize, n_trials: usize) -> f64 {
    if n_agents == 0 || n_trials == 0 {
        return 0.0;
    }

    let chunk_size = n_trials.div_ceil(rayon::current_num_threads());

    let total_cancel: f64 = (0..rayon::current_num_threads())
        .into_par_iter()
        .map(|thread_id| {
            let mut rng = Xorshift128Plus::new(
                (thread_id as u64)
                    .wrapping_mul(0xDEAD_BEEF_CAFE_BABE)
                    .wrapping_add(42),
            );
            let trials_this_thread =
                chunk_size.min(n_trials.saturating_sub(thread_id * chunk_size));
            let mut local_sum = 0.0f64;
            let mut buf = vec![0i8; n_agents];

            for _ in 0..trials_this_thread {
                // Generate + sum in one pass — cache-friendly
                let mut s: i64 = 0;
                for slot in &mut buf {
                    let val = rng.ternary();
                    *slot = val;
                    s += val as i64;
                }
                local_sum += 1.0 - (s.unsigned_abs() as f64 / n_agents as f64);
            }
            local_sum
        })
        .sum();

    total_cancel / n_trials as f64
}

// ─────────────────────────────────────────────────────────────
// Shannon Entropy
// ─────────────────────────────────────────────────────────────

/// Shannon entropy (in bits) of a ternary signal distribution.
///
/// Computes H = −Σ pᵢ log₂ pᵢ over the three symbols {−1, 0, +1}.
/// Maximum is log₂(3) ≈ 1.585 bits for a uniform distribution.
pub fn shannon_entropy(signals: &[i8]) -> f64 {
    let n = signals.len();
    if n == 0 {
        return 0.0;
    }

    let mut counts = [0u64; 3]; // [-1, 0, +1]

    for &s in signals {
        let idx = if s < 0 {
            0
        } else if s == 0 {
            1
        } else {
            2
        };
        counts[idx] += 1;
    }

    let nf = n as f64;
    let mut h = 0.0;
    for &c in &counts {
        if c > 0 {
            let p = c as f64 / nf;
            h -= p * (p.ln() / LN_2);
        }
    }
    h
}

// ─────────────────────────────────────────────────────────────
// Haar Wavelet Decomposition
// ─────────────────────────────────────────────────────────────

/// Haar wavelet decomposition of a ternary signal.
///
/// Returns `(approx, detail)` where:
/// - `approx` = running average (low-pass) coefficients
/// - `detail` = running difference (high-pass) coefficients
///
/// Demonstrates how the conservation law manifests in the frequency
/// domain: the detail coefficients capture fleet cancellation noise.
pub fn haar_decompose(signal: &[i8]) -> (Vec<f64>, Vec<f64>) {
    let n = signal.len();
    if n < 2 {
        let approx: Vec<f64> = signal.iter().map(|&s| s as f64).collect();
        let detail = vec![0.0; approx.len()];
        return (approx, detail);
    }

    let half = n / 2;
    let mut approx = Vec::with_capacity(half);
    let mut detail = Vec::with_capacity(half);

    let inv_sqrt2 = std::f64::consts::FRAC_1_SQRT_2;

    for pair in signal.chunks(2) {
        let a = pair[0] as f64;
        let b = if pair.len() > 1 { pair[1] as f64 } else { 0.0 };
        approx.push(inv_sqrt2 * (a + b));
        detail.push(inv_sqrt2 * (a - b));
    }

    (approx, detail)
}

// ─────────────────────────────────────────────────────────────
// Conservation Identity: γ + η = C
// ─────────────────────────────────────────────────────────────

/// Compute the conservation identity components.
///
/// Given signal vectors **x** (observed) and **g** (guide/channel):
/// - **γ** = I(x; g) — mutual information (shared structure)
/// - **η** = H(x|g) — conditional entropy (private noise)
/// - **C** = H(x) — total Shannon entropy
///
/// By the chain rule of information theory: γ + η = C, always.
///
/// Returns `(gamma, eta, c)`.
pub fn conservation_identity(x: &[i8], g: &[i8]) -> (f64, f64, f64) {
    let n = x.len().min(g.len());
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }

    // H(x) — total entropy
    let h_x = shannon_entropy(&x[..n]);

    // H(g) — guide entropy
    let h_g = shannon_entropy(&g[..n]);

    // H(x, g) — joint entropy
    let mut joint_counts = std::collections::HashMap::with_capacity(9);
    for i in 0..n {
        let key = ((x[i] + 1) as u8, (g[i] + 1) as u8);
        *joint_counts.entry(key).or_insert(0u64) += 1;
    }
    let nf = n as f64;
    let mut h_xg = 0.0;
    for &c in joint_counts.values() {
        let p = c as f64 / nf;
        h_xg -= p * (p.ln() / LN_2);
    }

    // I(x; g) = H(x) + H(g) − H(x, g)
    let gamma = h_x + h_g - h_xg;

    // H(x|g) = H(x, g) − H(g)
    let eta = h_xg - h_g;

    // C = H(x)
    let c = h_x;

    (gamma, eta, c)
}

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_decreases() {
        let d5 = delta(5);
        let d100 = delta(100);
        assert!(d100 < d5, "delta should decrease with n");
    }

    #[test]
    fn test_efficiency_bounds() {
        for &n in &[5, 10, 100, 1000, 10000] {
            let e = efficiency(n);
            assert!(e > 0.0 && e < 1.0, "efficiency in (0,1) for n={}", n);
        }
    }

    #[test]
    fn test_fleet_cancellation_perfect() {
        let signals = [-1i8, 1, -1, 1, 0];
        let c = fleet_cancellation(&signals);
        assert!((c - 1.0).abs() < 1e-9, "balanced fleet → 1.0");
    }

    #[test]
    fn test_fleet_cancellation_all_same() {
        let signals = [1i8; 10];
        let c = fleet_cancellation(&signals);
        assert!((c - 0.0).abs() < 1e-9, "all same → 0.0");
    }

    #[test]
    fn test_monte_carlo_approximates_theory() {
        let empirical = monte_carlo(1000, 5000);
        let theory = efficiency(1000);
        let err = ((empirical - theory) / theory).abs() * 100.0;
        assert!(err < 5.0, "MC error {}% should be < 5%", err);
    }

    #[test]
    fn test_shannon_entropy_uniform() {
        let signals = [-1i8, 0, 1, -1, 0, 1, -1, 0, 1];
        let h = shannon_entropy(&signals);
        assert!(
            (h - LOG2_3).abs() < 1e-9,
            "uniform ternary → log₂(3), got {}",
            h
        );
    }

    #[test]
    fn test_shannon_entropy_deterministic() {
        let signals = [1i8; 100];
        let h = shannon_entropy(&signals);
        assert!(h.abs() < 1e-9, "all same → 0 entropy, got {}", h);
    }

    #[test]
    fn test_haar_lengths() {
        let signal = [1i8, -1, 1, -1, 1, -1, 1, -1];
        let (approx, detail) = haar_decompose(&signal);
        assert_eq!(approx.len(), 4);
        assert_eq!(detail.len(), 4);
    }

    #[test]
    fn test_conservation_identity_holds() {
        let x = [1i8, -1, 0, 1, -1, 0, 1, -1, 0, 1];
        let g = [1i8, 1, 0, 1, -1, 0, 1, -1, 0, 1];
        let (gamma, eta, c) = conservation_identity(&x, &g);
        assert!(
            ((gamma + eta) - c).abs() < 1e-9,
            "γ + η = C: {} + {} = {} vs C = {}",
            gamma,
            eta,
            gamma + eta,
            c
        );
    }

    #[test]
    fn test_xorshift_terminates() {
        let mut rng = Xorshift128Plus::new(42);
        for _ in 0..1000 {
            let v = rng.ternary();
            assert!((-1..=1).contains(&v));
        }
    }

    #[test]
    fn test_ternary_distribution_uniform() {
        let mut rng = Xorshift128Plus::new(42);
        let mut counts = [0u64; 3];
        const N: u64 = 1_000_000;
        for _ in 0..N {
            let v = rng.ternary();
            let idx = (v + 1) as usize;
            counts[idx] += 1;
        }
        let expected = N as f64 / 3.0;
        for (label, &c) in [(-1, &counts[0]), (0, &counts[1]), (1, &counts[2])] {
            let deviation = (c as f64 - expected).abs() / expected;
            assert!(
                deviation < 0.01,
                "symbol {} deviated {:.4}% from uniform (count {})",
                label,
                deviation * 100.0,
                c
            );
        }
    }
}
