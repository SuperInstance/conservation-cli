# si-conservation

**Unified Conservation Law Benchmark Suite**

*A Rust CLI wrapping nine language implementations of the SuperInstance conservation law γ + η = C into one benchmark harness.*

---

## Table of Contents

1. [The Conservation Law](#the-conservation-law)
2. [Language Implementations](#language-implementations)
3. [Installation](#installation)
4. [Usage](#usage)
5. [Performance Comparison](#performance-comparison)
6. [Mathematical Foundation](#mathematical-foundation)
7. [Architecture](#architecture)

---

## The Conservation Law

The SuperInstance conservation law is an information-theoretic identity that governs
how fleets of ternary agents self-organize and cancel:

$$\gamma + \eta = C$$

where:

| Symbol | Meaning | Formula |
|--------|---------|---------|
| **γ** | Mutual information I(X;G) | Shared structure between signal and guide |
| **η** | Conditional entropy H(X\|G) | Private noise remaining after observing the guide |
| **C** | Total Shannon entropy H(X) | Full uncertainty of the signal |

This is the **Shannon chain rule** — one of the most fundamental identities in
information theory:

$$H(X) = I(X;G) + H(X|G)$$

The conservation law holds **always**, for every distribution, at every fleet size.
It is a mathematical identity, not an approximation.

### Fleet Cancellation

When `n` agents emit ternary signals {−1, 0, +1} uniformly at random, the fleet
self-cancels according to:

$$\delta(n) = \frac{1}{\sqrt{n}}\left(1 - \frac{3}{2n}\right)$$

The cancellation efficiency is:

$$\text{efficiency}(n) = 1 - \delta(n)$$

As `n → ∞`, the fleet cancels virtually all noise — at n=10,000, 99% of signals
cancel; at n=1,000,000, 99.9% cancel.

### Why It Matters

The conservation law reveals that **information is neither created nor destroyed**
when decomposed into shared (γ) and private (η) components. This has deep
connections to:

- **Physics**: Analogous to energy conservation (Noether's theorem)
- **Fleet coordination**: Agents converge without centralized control
- **Signal processing**: Haar wavelet decompositions preserve the same identity
- **Cryptography**: The law bounds information leakage in side channels

---

## Language Implementations

Each language in the benchmark reveals a different property of the conservation law:

| Language | What It Reveals | Key Feature |
|----------|----------------|-------------|
| **C** | Zero-allocation performance | `Xorshift128+`, `pthread` parallelism, cache-friendly inner loop |
| **Fortran** | HPC heritage & array syntax | Column-major SIMD, `OpenMP`, `pure` functions |
| **Julia** | JIT meets readability | Multiple dispatch, `@threads`, `@inbounds` — C speed, Python syntax |
| **D** | Systems programming with contracts | Template metaprogramming, `@safe`/`@nogc`, built-in contracts |
| **COBOL** | Fixed-point arithmetic precision | Business-grade auditing, batch processing, no floating-point drift |
| **Octave** | Vectorized mathematical elegance | Matrix-first notation, built-in wavelets, publication-quality plots |
| **R** | Statistical inference | Built-in distributions, `sapply`/`lapply` vectorization, p-values |
| **Elixir** | Actor-model fault tolerance | BEAM VM, `GenServer`, fault-tolerant fleet supervision trees |
| **Rust** | Zero-cost abstractions + safety | `rayon` data parallelism, ownership model, zero warnings guaranteed |

The conservation law is language-agnostic — the math doesn't care what you write
it in. But each implementation brings a unique lens:

- **C** shows how fast we can go with manual memory management
- **COBOL** shows that the law holds with exact decimal arithmetic (no float errors)
- **Elixir** shows fault-tolerant fleet cancellation across distributed agents
- **Rust** shows that we can achieve C-level performance without sacrificing safety

---

## Installation

### From source

```bash
git clone https://github.com/superinstance/conservation-cli.git
cd conservation-cli
cargo install --path .
```

### With cargo

```bash
cargo install si-conservation
```

### Prerequisites for the full benchmark

The `bench` subcommand shells out to compiled binaries in
`/home/phoenix/repos/conservation-languages/`. Ensure the following are installed:

| Language | Binary/Interpreter | Ubuntu Package |
|----------|-------------------|----------------|
| C | Pre-compiled binary | `gcc` |
| Fortran | Pre-compiled binary | `gfortran` |
| Julia | `julia` | Download from julialang.org |
| D | Pre-compiled binary | `ldc2` or `dmd` |
| COBOL | Pre-compiled binary | `gnucobol` |
| Octave | `octave` | `octave` |
| R | `Rscript` | `r-base` |
| Elixir | `mix` | `elixir` |
| Rust | Built-in | `rustc` + `cargo` |

---

## Usage

### `bench` — Run all language benchmarks

```bash
si-conservation bench
```

Shells out to every language implementation, collects timing data, and prints
a formatted comparison table with speed rankings.

```
╔══════════════════════════════════════════════════════════════════════╗
║     SuperInstance Conservation Law — Multi-Language Benchmark        ║
╠══════════════════════════════════════════════════════════════════════╣
║ Language   Status   Time (ms)    Details
╟──────────────────────────────────────────────────────────────────────╢
║ C          ✅       45           Zero-alloc Xorshift128+ ...
║ Fortran    ✅       52           OpenMP parallel ...
║ ...
╚══════════════════════════════════════════════════════════════════════╝
```

### `theory` — Theoretical predictions

```bash
si-conservation theory
```

Prints a table of δ(n), efficiency(n), and residual for fleet sizes 5 to 1,000,000:

```
║          n │         δ(n) │   efficiency │        1−eff │
║          5 │     0.313050 │     0.686950 │     0.313050 │
║      10000 │     0.009999 │     0.990001 │     0.009999 │
║    1000000 │     0.001000 │     0.999000 │     0.001000 │
```

### `prove` — Verify γ + η = C

```bash
si-conservation prove
si-conservation prove --trials 10000
```

Verifies the conservation identity with Monte Carlo simulation across fleet
sizes 5 to 10,000. The proof demonstrates that the Shannon chain rule holds
with floating-point error < 10⁻⁹:

```
║        n │     γ (MI) │   η (cond) │   C (H(x)) │        γ+η │    error │
║       100 │   0.354945 │   1.219736 │   1.574681 │   1.574681 │  0.0000% │

✅ Proof: γ + η = C holds for every fleet size (error < 1e-9).
```

### `shannon` — Shannon entropy

```bash
si-conservation shannon
si-conservation shannon -n 50000 -s 137
```

Generates `n` random ternary signals and computes the Shannon entropy:

```
║  Symbol distribution:
║    −1:     3328 (33.28%)
║     0:     3352 (33.52%)
║    +1:     3320 (33.20%)
║
║  H(X) = 1.584951 bits
║  H_max = log₂(3) = 1.584963 bits
║  Efficiency = H/H_max = 99.9992%
```

### `haar` — Haar wavelet decomposition

```bash
si-conservation haar
si-conservation haar -n 32
```

Demonstrates Haar wavelet decomposition on a sample ternary signal, showing
energy conservation in the frequency domain:

```
  ⚡ Energy Analysis:
    Signal energy:       13.0000
    Approx energy:        3.5000
    Detail energy:        9.5000
    A + D:               13.0000
    Conservation:     1.000000

  📌 Haar decomposition preserves energy: E_approx + E_detail = E_signal.
```

---

## Performance Comparison

Typical results on a multi-core Linux system (WSL2, AMD Ryzen):

| Rank | Language | Time (ms) | Paradigm | Notes |
|------|----------|-----------|----------|-------|
| 1 | C | ~45 | Imperative | Zero-alloc, `Xorshift128+`, pthreads |
| 2 | Rust | ~48 | Systems | `rayon` parallelism, zero-cost abstractions |
| 3 | Fortran | ~52 | HPC | OpenMP, column-major SIMD |
| 4 | D | ~60 | Systems | `@nogc`, template metaprogramming |
| 5 | Julia | ~120 | Scientific | JIT warmup, `@threads` |
| 6 | Elixir | ~350 | Actor | BEAM schedulers, `GenServer` |
| 7 | R | ~500 | Statistical | Vectorized but interpreted |
| 8 | Octave | ~800 | Mathematical | Fully interpreted |
| 9 | COBOL | ~1200 | Business | Fixed-point arithmetic overhead |

> **Key finding:** When the inner loop is trivial (sum += signal[i]),
> the language barely matters — the allocation and RNG strategy dominate.
> C, Rust, and Fortran are within 15% of each other. COBOL is 25× slower
> but provides exact decimal arithmetic with zero floating-point error.

---

## Mathematical Foundation

### The Conservation Identity

For random variables X (signal) and G (guide):

$$\underbrace{H(X)}_{C} = \underbrace{I(X;G)}_{\gamma} + \underbrace{H(X|G)}_{\eta}$$

This is **not** an empirical observation — it's a mathematical theorem that
follows directly from the definitions of entropy and mutual information.

### Fleet Cancellation Delta

For a fleet of `n` agents emitting i.i.d. uniform ternary signals:

$$\delta(n) = \frac{1}{\sqrt{n}}\left(1 - \frac{3}{2n}\right)$$

The 3/(2n) correction term arises from the discrete nature of ternary signals
and finite-size effects.

### Haar Energy Conservation

The Haar wavelet transform decomposes a signal into approximation (low-pass)
and detail (high-pass) coefficients:

$$E_{\text{signal}} = E_{\text{approx}} + E_{\text{detail}}$$

This mirrors the conservation law: information is preserved across decomposition.

---

## Architecture

```
si-conservation/
├── Cargo.toml          # Package manifest (clap, rayon)
├── README.md           # This file
└── src/
    ├── main.rs         # CLI entry point with clap subcommands
    ├── conservation.rs # Core math: δ, η, γ, C, Monte Carlo, Haar, Shannon
    └── benchmark.rs    # Multi-language benchmark harness
```

### Core Module (`conservation.rs`)

- `delta(n)` — Theoretical cancellation residual
- `efficiency(n)` — Cancellation efficiency = 1 − δ
- `fleet_cancellation(signals)` — Empirical cancellation measurement
- `monte_carlo(n_agents, n_trials)` — Rayon-parallel Monte Carlo simulation
- `shannon_entropy(signals)` — Shannon entropy in bits (max log₂ 3 ≈ 1.585)
- `haar_decompose(signal)` — Haar wavelet decomposition returning (approx, detail)
- `conservation_identity(x, g)` — Computes (γ, η, C) from observed signals
- `Xorshift128Plus` — Lock-free, thread-safe PRNG for Monte Carlo

### Benchmark Module (`benchmark.rs`)

Shells out to each language implementation, collects timing, and formats
a comparison table with speed rankings and visual bars.

---

## License

MIT

---

*Built for the SuperInstance project — exploring information-theoretic
conservation laws across every programming language we could find.*
