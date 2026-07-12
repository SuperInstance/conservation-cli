//! # Benchmark Harness
//!
//! Shells out to every language implementation of the conservation law
//! and collects timing data into a unified comparison table.

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// A single benchmark result row.
#[derive(Debug, Clone)]
pub struct BenchResult {
    pub language: &'static str,
    #[allow(dead_code)]
    pub binary: String,
    pub elapsed_ms: u128,
    pub exit_code: i32,
    pub output_preview: String,
}

impl BenchResult {
    fn ok(language: &'static str, binary: &str, elapsed_ms: u128, output: &str) -> Self {
        let preview: String = output.lines().take(3).collect::<Vec<_>>().join("\n");
        Self {
            language,
            binary: binary.to_string(),
            elapsed_ms,
            exit_code: 0,
            output_preview: preview,
        }
    }

    fn fail(language: &'static str, binary: &str, elapsed_ms: u128, stderr: &str) -> Self {
        let preview: String = stderr.lines().take(2).collect::<Vec<_>>().join("\n");
        Self {
            language,
            binary: binary.to_string(),
            elapsed_ms,
            exit_code: 1,
            output_preview: preview,
        }
    }

    pub fn status_icon(&self) -> &'static str {
        if self.exit_code == 0 {
            "✅"
        } else {
            "❌"
        }
    }
}

/// Run a command with a timeout and capture output.
fn run_timed(language: &'static str, binary: &str, mut cmd: Command) -> BenchResult {
    let start = Instant::now();
    let output = cmd.output();
    let elapsed = start.elapsed().as_millis();

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                BenchResult::ok(language, binary, elapsed, &stdout)
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                BenchResult::fail(language, binary, elapsed, &stderr)
            }
        }
        Err(e) => BenchResult::fail(language, binary, elapsed, &e.to_string()),
    }
}

/// Paths to each language implementation.
fn lang_paths() -> [(&'static str, PathBuf); 8] {
    let base = PathBuf::from("/home/phoenix/repos/conservation-languages");
    [
        ("C", base.join("c/zero_alloc")),
        ("Fortran", base.join("fortran/conservation")),
        ("D", base.join("dlang/conservation")),
        ("COBOL", base.join("cobol/conservation")),
        ("Julia", base.join("julia/conservation.jl")),
        ("Octave", base.join("matlab/conservation_law.m")),
        ("R", base.join("r/conservation_law.R")),
        ("Elixir", base.join("elixir")),
    ]
}

/// Run all external language benchmarks.
pub fn run_all_external() -> Vec<BenchResult> {
    let paths = lang_paths();
    let mut results = Vec::with_capacity(9);

    // C
    {
        let (_, path) = &paths[0];
        results.push(run_timed(
            "C",
            path.to_str().unwrap_or("c/zero_alloc"),
            Command::new(path),
        ));
    }

    // Fortran
    {
        let (_, path) = &paths[1];
        results.push(run_timed(
            "Fortran",
            path.to_str().unwrap_or("fortran/conservation"),
            Command::new(path),
        ));
    }

    // D
    {
        let (_, path) = &paths[2];
        results.push(run_timed(
            "D",
            path.to_str().unwrap_or("dlang/conservation"),
            Command::new(path),
        ));
    }

    // COBOL
    {
        let (_, path) = &paths[3];
        results.push(run_timed(
            "COBOL",
            path.to_str().unwrap_or("cobol/conservation"),
            Command::new(path),
        ));
    }

    // Julia
    {
        let (_, path) = &paths[4];
        let mut cmd = Command::new("julia");
        cmd.arg(path);
        results.push(run_timed("Julia", "julia conservation.jl", cmd));
    }

    // Octave
    {
        let (_, path) = &paths[5];
        let mut cmd = Command::new("octave");
        cmd.arg("--no-gui").arg("--quiet").arg(path);
        results.push(run_timed(
            "Octave",
            "octave --no-gui conservation_law.m",
            cmd,
        ));
    }

    // R
    {
        let (_, path) = &paths[6];
        let mut cmd = Command::new("Rscript");
        cmd.arg(path);
        results.push(run_timed("R", "Rscript conservation_law.R", cmd));
    }

    // Elixir
    {
        let (_, path) = &paths[7];
        let mut cmd = Command::new("mix");
        cmd.arg("run").arg("bench/runner.exs").current_dir(path);
        results.push(run_timed("Elixir", "mix run bench/runner.exs", cmd));
    }

    // Rust (internal)
    results.push(run_rust_internal());

    results
}

/// Run the internal Rust benchmark.
fn run_rust_internal() -> BenchResult {
    use crate::conservation;

    let start = Instant::now();

    // Monte Carlo at fleet size 10000, 5000 trials
    let _empirical = conservation::monte_carlo(10_000, 5_000);

    // Theory
    let _theory = conservation::efficiency(10_000);

    // Conservation identity
    let mut rng = conservation::Xorshift128Plus::new(42);
    let n = 10_000;
    let x: Vec<i8> = (0..n).map(|_| rng.ternary()).collect();
    let g: Vec<i8> = (0..n).map(|_| rng.ternary()).collect();
    let (gamma, eta, c) = conservation::conservation_identity(&x, &g);

    let elapsed = start.elapsed().as_millis();

    let output = format!(
        "Rust (rayon, {} threads)\nMC(10000, 5000) = {:.4}\nTheory = {:.4} | γ={:.4} η={:.4} C={:.4}",
        rayon::current_num_threads(),
        _empirical,
        _theory,
        gamma,
        eta,
        c
    );

    BenchResult::ok("Rust", "si-conservation (internal)", elapsed, &output)
}

/// Print a formatted comparison table.
pub fn print_table(results: &[BenchResult]) {
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║     SuperInstance Conservation Law — Multi-Language Benchmark        ║");
    println!("╠══════════════════════════════════════════════════════════════════════╣");
    println!(
        "║ {:<10} {:<8} {:<12} {:<40} ║",
        "Language", "Status", "Time (ms)", "Details"
    );
    println!("╟──────────────────────────────────────────────────────────────────────╢");

    for r in results {
        let details: String = r
            .output_preview
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(38)
            .collect();
        println!(
            "║ {:<10} {:<8} {:<12} {:<40} ║",
            r.language,
            r.status_icon(),
            if r.elapsed_ms > 0 {
                format!("{}", r.elapsed_ms)
            } else {
                "N/A".to_string()
            },
            details,
        );
    }

    println!("╚══════════════════════════════════════════════════════════════════════╝");

    // Speed ranking
    let mut sorted: Vec<&BenchResult> = results.iter().filter(|r| r.exit_code == 0).collect();
    sorted.sort_by_key(|r| r.elapsed_ms);

    println!();
    println!("📊 Speed Ranking (fastest first):");
    for (i, r) in sorted.iter().enumerate() {
        let bar_len = 30usize.min(sorted[0].elapsed_ms.max(1) as usize);
        let _ = bar_len;
        let ratio = if r.elapsed_ms > 0 {
            r.elapsed_ms as f64 / sorted[0].elapsed_ms.max(1) as f64
        } else {
            1.0
        };
        let bar = "█".repeat((20.0 / ratio).round() as usize);
        println!(
            "  {}. {:<10} {:>6} ms  {}",
            i + 1,
            r.language,
            r.elapsed_ms,
            bar
        );
    }
}
