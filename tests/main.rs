//! Comprehensive Test Runner for Checkmate
//!
//! This binary provides a CLI interface for running different types of tests:
//! - Unit tests
//! - Integration tests
//! - End-to-end tests
//! - Performance benchmarks
//! - Coverage reports

use clap::{Parser, Subcommand};
use colored::*;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "checkmate-test-runner")]
#[command(about = "Comprehensive test runner for Checkmate", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all tests (unit, integration, E2E)
    All {
        /// Run E2E tests against deployed dev stack
        #[arg(long)]
        e2e: bool,

        /// Generate coverage report
        #[arg(long)]
        coverage: bool,

        /// Run in CI mode (longer timeouts, no interactive output)
        #[arg(long)]
        ci: bool,
    },

    /// Run only unit tests
    Unit {
        /// Run with verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run only integration tests
    Integration {
        /// Run with verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run only end-to-end tests
    E2e {
        /// Base URL for E2E tests
        #[arg(long, default_value = "http://localhost:3000")]
        base_url: String,

        /// Timeout in seconds
        #[arg(long, default_value = "60")]
        timeout: u64,
    },

    /// Run performance benchmarks
    Bench {
        /// Benchmark specific component
        #[arg(long)]
        component: Option<String>,
    },

    /// Generate coverage report
    Coverage {
        /// Output format (html, lcov, text)
        #[arg(long, default_value = "html")]
        format: String,

        /// Output directory for HTML reports
        #[arg(long, default_value = "target/coverage")]
        output_dir: String,
    },

    /// Deploy dev stack for E2E testing
    DeployDev,

    /// Clean test artifacts
    Clean,
}

#[derive(Debug)]
struct TestResults {
    total: usize,
    passed: usize,
    failed: usize,
    duration: std::time::Duration,
}

impl TestResults {
    fn new() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            duration: std::time::Duration::default(),
        }
    }

    fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passed as f64) / (self.total as f64) * 100.0
        }
    }

    fn is_successful(&self) -> bool {
        self.failed == 0
    }
}

fn run_command(cmd: &str, args: &[&str], description: &str) -> Result<TestResults, String> {
    println!("{} {}", "▶".green(), description);

    let start_time = Instant::now();
    let output = Command::new(cmd)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let duration = start_time.elapsed();

    let mut results = TestResults::new();
    results.duration = duration;

    if output.status.success() {
        results.passed = 1;
        results.total = 1;
        println!("{} {} ({:.2?})", "✓".green(), "Success".green(), duration);
    } else {
        results.failed = 1;
        results.total = 1;
        println!("{} {} ({:.2?})", "✗".red(), "Failed".red(), duration);
    }

    Ok(results)
}

fn run_cargo_test(filter: Option<&str>, test_type: &str) -> Result<TestResults, String> {
    let mut args = vec!["test"];

    if let Some(filter) = filter {
        args.push(filter);
    }

    args.extend_from_slice(&["--", "--test-threads=1"]);

    run_command("cargo", &args, &format!("Running {} tests", test_type))
}

fn run_unit_tests(verbose: bool) -> Result<TestResults, String> {
    println!("\n{}", "🧪 Running Unit Tests".bold());

    let mut args = vec!["test", "--lib", "--bins"];
    if verbose {
        args.push("--verbose");
    }

    run_command("cargo", &args, "Unit tests")
}

fn run_integration_tests(verbose: bool) -> Result<TestResults, String> {
    println!("\n{}", "🔗 Running Integration Tests".bold());

    let mut args = vec!["test", "--test", "*"];
    if verbose {
        args.push("--verbose");
    }

    run_command("cargo", &args, "Integration tests")
}

fn run_e2e_tests(base_url: &str, timeout: u64) -> Result<TestResults, String> {
    println!("\n{}", "🌐 Running End-to-End Tests".bold());

    // Set environment variables for E2E tests
    std::env::set_var("TEST_BASE_URL", base_url);
    std::env::set_var("TEST_TIMEOUT_SECONDS", timeout.to_string());

    run_cargo_test(Some("e2e_tests"), "E2E")
}

fn run_benchmarks(component: Option<&str>) -> Result<TestResults, String> {
    println!("\n{}", "📊 Running Benchmarks".bold());

    let mut args = vec!["bench"];
    if let Some(comp) = component {
        args.push("--bench");
        args.push(comp);
    }

    run_command("cargo", &args, "Performance benchmarks")
}

fn generate_coverage(format: &str, output_dir: &str) -> Result<TestResults, String> {
    println!("\n{}", "📈 Generating Coverage Report".bold());

    let mut args = vec!["llvm-cov", "--workspace"];

    match format {
        "html" => {
            args.extend_from_slice(&["--html", "--output-dir", output_dir]);
            run_command(
                "cargo",
                &args,
                &format!("HTML coverage report to {}", output_dir),
            )
        }
        "lcov" => {
            args.extend_from_slice(&["--lcov", "--output-path", "lcov.info"]);
            run_command("cargo", &args, "LCov coverage report")
        }
        "text" => {
            args.push("--text");
            run_command("cargo", &args, "Text coverage report")
        }
        _ => Err(format!("Unsupported coverage format: {}", format)),
    }
}

fn deploy_dev_stack() -> Result<TestResults, String> {
    println!("\n{}", "🚀 Deploying Development Stack".bold());

    run_command("npm", &["run", "deploy:dev"], "Deploying dev stack")
}

fn clean_test_artifacts() -> Result<TestResults, String> {
    println!("\n{}", "🧹 Cleaning Test Artifacts".bold());

    run_command("cargo", &["clean"], "Cleaning cargo artifacts")
}

fn run_all_tests(e2e: bool, coverage: bool, ci: bool) -> Result<TestResults, String> {
    println!("{}", "🎯 Running Complete Test Suite".bold().blue());

    if ci {
        println!("{}", "🤖 CI mode enabled".yellow());
        std::env::set_var("CI", "true");
    }

    let mut total_results = TestResults::new();

    // Unit tests
    match run_unit_tests(false) {
        Ok(results) => {
            total_results.total += results.total;
            total_results.passed += results.passed;
            total_results.failed += results.failed;
            total_results.duration += results.duration;
        }
        Err(e) => {
            eprintln!("{} Unit tests failed: {}", "✗".red(), e);
            total_results.failed += 1;
            total_results.total += 1;
        }
    }

    // Integration tests
    match run_integration_tests(false) {
        Ok(results) => {
            total_results.total += results.total;
            total_results.passed += results.passed;
            total_results.failed += results.failed;
            total_results.duration += results.duration;
        }
        Err(e) => {
            eprintln!("{} Integration tests failed: {}", "✗".red(), e);
            total_results.failed += 1;
            total_results.total += 1;
        }
    }

    // E2E tests (optional)
    if e2e {
        match run_e2e_tests("http://localhost:3000", 60) {
            Ok(results) => {
                total_results.total += results.total;
                total_results.passed += results.passed;
                total_results.failed += results.failed;
                total_results.duration += results.duration;
            }
            Err(e) => {
                eprintln!("{} E2E tests failed: {}", "✗".red(), e);
                total_results.failed += 1;
                total_results.total += 1;
            }
        }
    }

    // Coverage (optional)
    if coverage {
        match generate_coverage("text", "target/coverage") {
            Ok(results) => {
                total_results.total += results.total;
                total_results.passed += results.passed;
                total_results.failed += results.failed;
                total_results.duration += results.duration;
            }
            Err(e) => {
                eprintln!("{} Coverage generation failed: {}", "✗".red(), e);
                total_results.failed += 1;
                total_results.total += 1;
            }
        }
    }

    Ok(total_results)
}

fn print_summary(results: &TestResults) {
    println!("\n{}", "📊 Test Summary".bold());
    println!("{}", "─".repeat(50));
    println!("Total Tests:    {}", results.total);
    println!("{} Passed:      {}", "✅".green(), results.passed);
    println!("{} Failed:      {}", "❌".red(), results.failed);
    println!("Success Rate:   {:.1}%", results.success_rate());
    println!("Duration:       {:.2?}", results.duration);
    println!("{}", "─".repeat(50));

    if results.is_successful() {
        println!("{}", "🎉 All tests passed!".bold().green());
    } else {
        println!("{}", "💥 Some tests failed!".bold().red());
    }
}

fn main() {
    let cli = Cli::parse();

    let results = match cli.command {
        Commands::All { e2e, coverage, ci } => run_all_tests(e2e, coverage, ci),
        Commands::Unit { verbose } => run_unit_tests(verbose),
        Commands::Integration { verbose } => run_integration_tests(verbose),
        Commands::E2e { base_url, timeout } => run_e2e_tests(&base_url, timeout),
        Commands::Bench { component } => run_benchmarks(component.as_deref()),
        Commands::Coverage { format, output_dir } => generate_coverage(&format, &output_dir),
        Commands::DeployDev => deploy_dev_stack(),
        Commands::Clean => clean_test_artifacts(),
    };

    match results {
        Ok(results) => {
            print_summary(&results);
            if !results.is_successful() {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{} {}", "💥 Error:".bold().red(), e);
            std::process::exit(1);
        }
    }
}
