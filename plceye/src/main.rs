//! plceye - PLC Code Smell Detector CLI

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use plceye::{SmellConfig, SmellDetector, Report};

#[derive(Parser)]
#[command(name = "plceye")]
#[command(version, about = "PLC code smell detector for L5X files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// L5X files to analyze
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Configuration file (default: plceye.toml if exists)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Minimum severity to report: info, warning, error
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    severity: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a default plceye.toml configuration file
    Init,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(Commands::Init) = cli.command {
        return init_config();
    }

    // Check for input files
    if cli.files.is_empty() {
        eprintln!("Error: No input files specified");
        eprintln!("Usage: plceye <FILE>...");
        eprintln!("Try 'plceye --help' for more information.");
        return ExitCode::from(1);
    }

    // Load or create configuration
    let mut config = if let Some(ref path) = cli.config {
        match SmellConfig::from_file(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config: {}", e);
                return ExitCode::from(1);
            }
        }
    } else if Path::new("plceye.toml").exists() {
        match SmellConfig::from_file(Path::new("plceye.toml")) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to load plceye.toml: {}", e);
                SmellConfig::default()
            }
        }
    } else {
        SmellConfig::default()
    };

    // Apply severity from CLI
    config.general.min_severity = cli.severity.clone();

    let detector = SmellDetector::with_config(config);
    let min_severity = detector.min_severity();

    // Collect all results
    let mut all_reports: Vec<(String, Report)> = Vec::new();
    let mut has_errors = false;

    for file in &cli.files {
        match detector.analyze_file(file) {
            Ok(report) => {
                all_reports.push((file.display().to_string(), report));
            }
            Err(e) => {
                eprintln!("Error analyzing {}: {}", file.display(), e);
                has_errors = true;
            }
        }
    }

    // Calculate total issues
    let total_issues: usize = all_reports
        .iter()
        .map(|(_, r)| r.filter_by_severity(min_severity).len())
        .sum();

    // Output results
    for (file, report) in &all_reports {
        let filtered = report.filter_by_severity(min_severity);
        if !filtered.is_empty() {
            println!("\n=== {} ===", file);
            for smell in filtered {
                println!("{}", smell);
            }
        }
    }

    println!();
    if total_issues == 0 {
        println!("No issues found in {} file(s).", cli.files.len());
    } else {
        println!("Found {} issue(s) in {} file(s).", total_issues, cli.files.len());
    }

    if has_errors {
        ExitCode::from(2)
    } else if total_issues > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn init_config() -> ExitCode {
    let path = Path::new("plceye.toml");
    if path.exists() {
        eprintln!("Error: plceye.toml already exists");
        return ExitCode::from(1);
    }

    let content = SmellConfig::default_toml();
    match std::fs::write(path, content) {
        Ok(_) => {
            println!("Created plceye.toml with default configuration");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error writing plceye.toml: {}", e);
            ExitCode::from(1)
        }
    }
}
