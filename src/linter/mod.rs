mod utils;
use std::io::{self, Write};
use std::path::Path;
use utils::clean_exit;

use crate::LintArgs;
use std::process::Command;
use std::{fs, path::PathBuf};
use yaml_rust2::{Yaml, YamlLoader};
use colored::*;

/// Linter processing for workflows
pub fn lint(args: LintArgs) {
    println!("{}", "-------------------------------".blue());
    println!("{}", "-------- STARTING LINTER ------".blue().bold());
    println!("{}", "-------------------------------".blue());

    let config_path = args.base_path.join(&args.config_file);
    println!(
        "{} {}",
        "[INFO] Using config file:".blue(),
        config_path.display()
    );

    let config_str = fs::read_to_string(&config_path)
        .unwrap_or_else(|err| clean_exit(&format!("{}", format!("[ERROR] Failed to read config file: {}", err).red())));

    let configs = YamlLoader::load_from_str(&config_str)
        .unwrap_or_else(|err| clean_exit(&format!("{}", format!("[ERROR] Invalid YAML syntax: {}", err).red())));

    let config = configs
        .first()
        .unwrap_or_else(|| clean_exit(&format!("{}", "[ERROR] Config file is empty.".red())));

    let manifest_errors = match config["manifests"].as_vec() {
        Some(manifest_paths) => {
            println!(
                "{} {}",
                "[INFO] Found manifests to lint:".blue(),
                manifest_paths.len()
            );
            manifests(&args.base_path, manifest_paths)
        }
        None => {
            println!("{}", "[WARN] No manifests defined in config... Skipping linting.".yellow());
            Vec::new()
        }
    };

    if !manifest_errors.is_empty() {
        eprintln!("\n{}", "[ERROR] Linting completed with issues:".red().bold());
        for error in manifest_errors {
            eprintln!("  {} {}", "•".red(), error);
        }
        clean_exit(&format!("{}", "[FAIL] Linting failed due to errors.".red().bold()))
    } else {
        println!(
            "\n{}",
            "[SUCCESS] All manifests linted successfully. No issues found."
                .green()
                .bold()
        );
    }
}

/// Run the linter against conventional, non-helm-based workflows
fn manifests(base_path: &Path, manifests: &[Yaml]) -> Vec<String> {
    println!("{}", "[INFO] Beginning linting for each manifest...".blue());

    manifests.iter().for_each(|manifest| {
        if let Some(path_str) = manifest.as_str() {
            println!("  {} {}", "→".cyan(), path_str.cyan());
        }
    });

    let abs_manifest_paths: Vec<PathBuf> = manifests
        .iter()
        .map(|manifest| base_path.join(manifest.as_str().unwrap()))
        .collect();

    let mut errors: Vec<String> = Vec::new();

    for manifest_path in &abs_manifest_paths {
        println!(
            "{} {}",
            "[INFO] Running `argo lint` on:".blue(),
            manifest_path.display()
        );

        let output = Command::new("argo")
            .arg("lint")
            .arg(manifest_path)
            .arg("--offline")
            .arg("--output")
            .arg("simple")
            .output()
            .unwrap_or_else(|err| {
                clean_exit(&format!(
                    "{}",
                    format!(
                        "[ERROR] Failed to execute `argo lint` on {}: {}",
                        manifest_path.display(),
                        err
                    )
                    .red()
                ))
            });

        if output.status.success() {
            println!("  {} {}", "[PASS]".green(), manifest_path.display());
        } else {
            let err_msg = String::from_utf8_lossy(&output.stdout);
            let sub_errors: Vec<String> = err_msg
                .lines()
                .filter(|msg| {
                    if msg.contains("couldn't find cluster workflow template") {
                        eprintln!(
                            "  {} {}",
                            "[WARN]".yellow(),
                            "External template reference skipped:"
                        );
                        eprintln!("     {}", msg.yellow());
                        false
                    } else {
                        true
                    }
                })
                .map(str::to_owned)
                .collect();

            if !sub_errors.is_empty() {
                println!("  {} {}", "[FAIL]".red(), manifest_path.display());
                errors.extend(sub_errors);
            }
        }
    }

    errors
}
