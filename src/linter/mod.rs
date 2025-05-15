mod utils;
use std::path::Path;
use utils::clean_exit;

use crate::LintArgs;
use std::process::Command;
use std::{fs, path::PathBuf};
use yaml_rust2::{Yaml, YamlLoader};

/// Linter processing for workflows
pub fn lint(args: LintArgs) {
    println!("-------------------------------");
    println!("--------STARTING LINTER--------");
    println!("-------------------------------");
    let config_path = args.base_path.join(&args.config_file);

    let config_str = fs::read_to_string(&config_path)
        .unwrap_or_else(|err| clean_exit(&format!("Failed to read config: {}", err)));

    let configs = YamlLoader::load_from_str(&config_str)
        .unwrap_or_else(|err| clean_exit(&format!("Invalid YAML: {}", err)));

    let config = configs
        .first()
        .unwrap_or_else(|| clean_exit("Empty config file."));

    let manifest_errors = match config["manifests"].as_vec() {
        Some(manifest_paths) => manifests(&args.base_path, manifest_paths),
        None => {
            println!("No manifests to lint... Skipping");
            Vec::new()
        }
    };

    if !manifest_errors.is_empty() {
        eprintln!("Linting completed with errors:");
        for error in manifest_errors {
            eprintln!("- {}", error);
        }
        clean_exit("Linting Failed")
    } else {
        println!("All manifests linted successfully.");
    }
}

/// Run the linter against conventional, non-helm-based workflows
fn manifests(base_path: &Path, manifests: &[Yaml]) -> Vec<String> {
    println!("Found the following manifests...");
    manifests
        .iter()
        .for_each(|manifest| println!("{}", manifest.as_str().unwrap()));

    let abs_manifest_paths: Vec<PathBuf> = manifests
        .iter()
        .map(|manifest| base_path.join(manifest.as_str().unwrap()))
        .collect();

    let mut errors: Vec<String> = Vec::new();

    for manifest_path in &abs_manifest_paths {
        let output = Command::new("argo")
            .arg("lint")
            .arg(manifest_path)
            .arg("--offline")
            .arg("--output")
            .arg("simple")
            .output()
            .unwrap();

        // println!("{:#?}", output);
        match output.status.success() {
            true => {}
            _ => {
                let err_msg = String::from_utf8(output.stdout).unwrap();
                let sub_errors: Vec<String> = err_msg
                    .lines()
                    .filter(|msg| {
                        if msg.contains("couldn't find cluster workflow template") {
                            println!(
                                "WARNING: Unable to lint templates referencing external templates:"
                            );
                            false
                        } else {
                            true
                        }
                    })
                    .map(str::to_owned)
                    .collect();

                errors.extend(sub_errors);
            }
        };
    }
    errors
}
