//! plceye - PLC Code Rule Detector CLI

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use plceye::{RuleConfig, RuleDetector, Report, ParseStats};

#[derive(Parser)]
#[command(name = "plceye")]
#[command(version, about = "PLC code rule detector for L5X files", long_about = None)]
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

    /// Show file statistics only (no rule detection)
    #[arg(long)]
    stats: bool,
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

    // Handle --stats mode
    if cli.stats {
        return show_stats(&cli.files);
    }

    // Load or create configuration
    let mut config = if let Some(ref path) = cli.config {
        match RuleConfig::from_file(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config: {}", e);
                return ExitCode::from(1);
            }
        }
    } else if Path::new("plceye.toml").exists() {
        match RuleConfig::from_file(Path::new("plceye.toml")) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Failed to load plceye.toml: {}", e);
                RuleConfig::default()
            }
        }
    } else {
        RuleConfig::default()
    };

    // Apply severity from CLI
    config.general.min_severity = cli.severity.clone();

    let detector = RuleDetector::with_config(config);
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
            for rule in filtered {
                println!("{}", rule);
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

    let content = RuleConfig::default_toml();
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

fn show_stats(files: &[PathBuf]) -> ExitCode {
    let detector = RuleDetector::new();
    let mut has_errors = false;

    for file in files {
        println!("=== {} ===", file.display());
        
        // Try to load the project to detect format
        match plceye::LoadedProject::from_file(file) {
            Ok(project) => {
                if project.format == plceye::FileFormat::PlcOpen {
                    // PLCopen format - show PLCopen stats
                    match detector.get_plcopen_stats(&project) {
                        Ok(stats) => {
                            print_plcopen_stats(&stats);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            has_errors = true;
                        }
                    }
                } else {
                    // L5X format - show L5X stats
                    match detector.get_stats(&project) {
                        Ok(stats) => {
                            print_stats(&stats);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            has_errors = true;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                has_errors = true;
            }
        }
        println!();
    }

    if has_errors {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_stats(stats: &ParseStats) {
    println!("Programs:           {:>6}", stats.programs);
    println!("AOIs:               {:>6}", stats.aois);
    println!("Routines:           {:>6}", stats.routines);
    println!();
    println!("RLL Rungs (total):  {:>6}", stats.rungs);
    println!("  In programs:      {:>6}", stats.rll_rungs_programs);
    println!("  In AOIs:          {:>6}", stats.rll_rungs_aois);
    println!("  Parsed OK:        {:>6}", stats.parsed_ok);
    println!("  Parse errors:     {:>6}", stats.parsed_err);
    println!();
    println!("ST Routines:        {:>6}", stats.st_routines);
    println!("  In programs:      {:>6}", stats.st_routines_programs);
    println!("  In AOIs:          {:>6}", stats.st_routines_aois);
    println!("  Parsed OK:        {:>6}", stats.st_parsed_ok);
    println!("  Parse errors:     {:>6}", stats.st_parsed_err);
    println!();
    println!("Tag references:     {:>6}", stats.tag_references);
    println!("Unique tags:        {:>6}", stats.unique_tags);
    
    // Only show complexity stats if there are ST routines
    if stats.st_parsed_ok > 0 {
        println!();
        println!("ST Complexity:");
        println!("  Max complexity:   {:>6}", stats.st_max_complexity);
        println!("  Avg complexity:   {:>6.1}", stats.st_avg_complexity);
        println!("  Max nesting:      {:>6}", stats.st_max_nesting);
        println!("  Avg nesting:      {:>6.1}", stats.st_avg_nesting);
    }
}

fn print_plcopen_stats(stats: &plceye::PlcopenStats) {
    println!("POUs (total):       {:>6}", stats.pous);
    println!("  Functions:        {:>6}", stats.functions);
    println!("  Function Blocks:  {:>6}", stats.function_blocks);
    println!("  Programs:         {:>6}", stats.programs);
    println!("  Empty POUs:       {:>6}", stats.empty_pous);
    println!();
    println!("Language Usage:");
    println!("  ST (Structured Text):     {:>6}", stats.st_bodies);
    println!("  IL (Instruction List):    {:>6}", stats.il_bodies);
    println!("  FBD (Function Block):     {:>6}", stats.fbd_bodies);
    println!("  LD (Ladder Diagram):      {:>6}", stats.ld_bodies);
    println!("  SFC (Sequential Chart):   {:>6}", stats.sfc_bodies);
    println!();
    println!("Variables:          {:>6}", stats.variables);
}
