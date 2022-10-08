// Inspired by Paul Woolcock's cargo-fmt (https://github.com/pwoolcoc/cargo-fmt/).

#![deny(warnings)]
#![allow(clippy::match_like_matches_macro)]

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use clap::{AppSettings, CommandFactory, Parser};

#[derive(Parser)]
#[clap(
    global_setting(AppSettings::NoAutoVersion),
    bin_name = "cargo doc-stripper",
    about = "This utility extract/regenerate doc comments into source code."
)]
pub struct Opts {
    /// No output printed to stdout
    #[clap(short = 'q', long = "quiet")]
    quiet: bool,

    /// Use verbose output
    #[clap(short = 'v', long = "verbose")]
    verbose: bool,

    /// Print doc-stripper version and exit
    #[clap(long = "version")]
    version: bool,

    /// Specify package to format
    #[clap(
        short = 'p',
        long = "package",
        value_name = "package",
        multiple_values = true
    )]
    packages: Vec<String>,

    /// Specify path to Cargo.toml
    #[clap(long = "manifest-path", value_name = "manifest-path")]
    manifest_path: Option<String>,

    /// Options passed to rustfmt
    // 'raw = true' to make `--` explicit.
    #[clap(name = "doc_stripper_options", raw(true))]
    doc_stripper_options: Vec<String>,

    /// Format all packages, and also their local path-based dependencies
    #[clap(long = "all")]
    format_all: bool,

    #[clap(long = "strip", short = 's')]
    strip: bool,

    #[clap(long = "regenerate", short = 'r')]
    regenerate: bool,
}

fn main() {
    let exit_status = execute();
    std::io::stdout().flush().unwrap();
    std::process::exit(exit_status);
}

const SUCCESS: i32 = 0;
const FAILURE: i32 = 1;

fn execute() -> i32 {
    // Drop extra `doc-stripper` argument provided by `cargo`.
    let mut found_doc_stripper = false;
    let args = env::args().filter(|x| {
        if found_doc_stripper {
            true
        } else {
            found_doc_stripper = x == "doc-stripper";
            x != "doc-stripper"
        }
    }).collect::<Vec<_>>();

    let opts = Opts::parse_from(&args);

    let verbosity = match (opts.verbose, opts.quiet) {
        (false, false) => Verbosity::Normal,
        (false, true) => Verbosity::Quiet,
        (true, false) => Verbosity::Verbose,
        (true, true) => {
            print_usage_to_stderr("quiet mode and verbose mode are not compatible");
            return FAILURE;
        }
    };

    if opts.version {
        return handle_command_status(get_doc_stripper_info(&[String::from("--version")]));
    }

    if opts.strip && opts.regenerate {
        print_usage_to_stderr("`--strip` and `--regenerate` are not compatible");
        return FAILURE;
    }

    let strategy = RustdocStripperStrategy::from_opts(&opts);

    if let Some(specified_manifest_path) = opts.manifest_path {
        if !specified_manifest_path.ends_with("Cargo.toml") {
            print_usage_to_stderr("the manifest-path must be a path to a Cargo.toml file");
            return FAILURE;
        }
        let manifest_path = PathBuf::from(specified_manifest_path);
        handle_command_status(run_doc_stripper(
            verbosity,
            &strategy,
            &opts.doc_stripper_options,
            Some(&manifest_path),
        ))
    } else {
        handle_command_status(run_doc_stripper(verbosity, &strategy, &opts.doc_stripper_options, None))
    }
}

fn doc_stripper_command() -> Command {
    let doc_stripper_var = env::var_os("DOC_STRIPPER");
    let doc_stripper = match &doc_stripper_var {
        Some(doc_stripper) => doc_stripper,
        None => OsStr::new("doc-stripper"),
    };
    Command::new(doc_stripper)
}

fn print_usage_to_stderr(reason: &str) {
    eprintln!("{}", reason);
    let app = Opts::command();
    app.after_help("")
        .write_help(&mut io::stderr())
        .expect("failed to write to stderr");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Verbose,
    Normal,
    Quiet,
}

fn handle_command_status(status: Result<i32, io::Error>) -> i32 {
    match status {
        Err(e) => {
            print_usage_to_stderr(&e.to_string());
            FAILURE
        }
        Ok(status) => status,
    }
}

fn get_doc_stripper_info(args: &[String]) -> Result<i32, io::Error> {
    let mut command = doc_stripper_command()
        .stdout(std::process::Stdio::inherit())
        .args(args)
        .spawn()
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => io::Error::new(
                io::ErrorKind::Other,
                "Could not run doc-stripper, please make sure it is in your PATH.",
            ),
            _ => e,
        })?;
    let result = command.wait()?;
    if result.success() {
        Ok(SUCCESS)
    } else {
        Ok(result.code().unwrap_or(SUCCESS))
    }
}

/// Target uses a `path` field for equality and hashing.
#[derive(Debug)]
pub struct Target {
    /// A path to the main source file of the target.
    path: PathBuf,
    /// A kind of target (e.g., lib, bin, example, ...).
    kind: String,
}

impl Target {
    pub fn from_target(target: &cargo_metadata::Target) -> Self {
        let path = PathBuf::from(&target.src_path);
        let canonicalized = fs::canonicalize(&path).unwrap_or(path);

        Target {
            path: canonicalized,
            kind: target.kind[0].clone(),
        }
    }
}

impl PartialEq for Target {
    fn eq(&self, other: &Target) -> bool {
        self.path == other.path
    }
}

impl PartialOrd for Target {
    fn partial_cmp(&self, other: &Target) -> Option<Ordering> {
        Some(self.path.cmp(&other.path))
    }
}

impl Ord for Target {
    fn cmp(&self, other: &Target) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl Eq for Target {}

impl Hash for Target {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RustdocStripperStrategy {
    /// Format every packages and dependencies.
    All,
    /// Format packages that are specified by the command line argument.
    Some(Vec<String>),
    /// Format the root packages only.
    Root,
}

impl RustdocStripperStrategy {
    pub fn from_opts(opts: &Opts) -> Self {
        match (opts.format_all, opts.packages.is_empty()) {
            (false, true) => Self::Root,
            (true, _) => Self::All,
            (false, false) => Self::Some(opts.packages.clone()),
        }
    }
}

/// Based on the specified `RustdocStripperStrategy`, returns a set of main source files.
fn get_targets(
    strategy: &RustdocStripperStrategy,
    manifest_path: Option<&Path>,
) -> Result<BTreeSet<Target>, io::Error> {
    let mut targets = BTreeSet::new();

    match *strategy {
        RustdocStripperStrategy::Root => get_targets_root_only(manifest_path, &mut targets)?,
        RustdocStripperStrategy::All => {
            get_targets_recursive(manifest_path, &mut targets, &mut BTreeSet::new())?
        }
        RustdocStripperStrategy::Some(ref hitlist) => {
            get_targets_with_hitlist(manifest_path, hitlist, &mut targets)?
        }
    }

    if targets.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to find targets".to_owned(),
        ))
    } else {
        Ok(targets)
    }
}

fn get_targets_root_only(
    manifest_path: Option<&Path>,
    targets: &mut BTreeSet<Target>,
) -> Result<(), io::Error> {
    let metadata = get_cargo_metadata(manifest_path)?;
    let workspace_root_path = PathBuf::from(&metadata.workspace_root).canonicalize()?;
    let (in_workspace_root, current_dir_manifest) = if let Some(target_manifest) = manifest_path {
        (
            workspace_root_path == target_manifest,
            target_manifest.canonicalize()?,
        )
    } else {
        let current_dir = env::current_dir()?.canonicalize()?;
        (
            workspace_root_path == current_dir,
            current_dir.join("Cargo.toml"),
        )
    };

    let package_targets = match metadata.packages.len() {
        1 => metadata.packages.into_iter().next().unwrap().targets,
        _ => metadata
            .packages
            .into_iter()
            .filter(|p| {
                in_workspace_root
                    || PathBuf::from(&p.manifest_path)
                        .canonicalize()
                        .unwrap_or_default()
                        == current_dir_manifest
            })
            .flat_map(|p| p.targets)
            .collect(),
    };

    for target in package_targets {
        targets.insert(Target::from_target(&target));
    }

    Ok(())
}

fn get_targets_recursive(
    manifest_path: Option<&Path>,
    targets: &mut BTreeSet<Target>,
    visited: &mut BTreeSet<String>,
) -> Result<(), io::Error> {
    let metadata = get_cargo_metadata(manifest_path)?;
    for package in &metadata.packages {
        add_targets(&package.targets, targets);

        // Look for local dependencies using information available since cargo v1.51
        // It's theoretically possible someone could use a newer version of doc-stripper with
        // a much older version of `cargo`, but we don't try to explicitly support that scenario.
        // If someone reports an issue with path-based deps not being formatted, be sure to
        // confirm their version of `cargo` (not `cargo-doc-stripper`) is >= v1.51
        // https://github.com/rust-lang/cargo/pull/8994
        for dependency in &package.dependencies {
            if dependency.path.is_none() || visited.contains(&dependency.name) {
                continue;
            }

            let manifest_path = PathBuf::from(dependency.path.as_ref().unwrap()).join("Cargo.toml");
            if manifest_path.exists()
                && !metadata
                    .packages
                    .iter()
                    .any(|p| p.manifest_path.eq(&manifest_path))
            {
                visited.insert(dependency.name.to_owned());
                get_targets_recursive(Some(&manifest_path), targets, visited)?;
            }
        }
    }

    Ok(())
}

fn get_targets_with_hitlist(
    manifest_path: Option<&Path>,
    hitlist: &[String],
    targets: &mut BTreeSet<Target>,
) -> Result<(), io::Error> {
    let metadata = get_cargo_metadata(manifest_path)?;
    let mut workspace_hitlist: BTreeSet<&String> = BTreeSet::from_iter(hitlist);

    for package in metadata.packages {
        if workspace_hitlist.remove(&package.name) {
            for target in package.targets {
                targets.insert(Target::from_target(&target));
            }
        }
    }

    if workspace_hitlist.is_empty() {
        Ok(())
    } else {
        let package = workspace_hitlist.iter().next().unwrap();
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("package `{}` is not a member of the workspace", package),
        ))
    }
}

fn add_targets(target_paths: &[cargo_metadata::Target], targets: &mut BTreeSet<Target>) {
    for target in target_paths {
        targets.insert(Target::from_target(target));
    }
}

fn run_doc_stripper(
    verbosity: Verbosity,
    strategy: &RustdocStripperStrategy,
    args: &[String],
    manifest_path: Option<&Path>,
) -> Result<i32, io::Error> {
    let targets = get_targets(strategy, manifest_path)?;

    if verbosity == Verbosity::Verbose {
        for target in &targets {
            println!("[{}] {:?}", target.kind, target.path)
        }
    }

    let mut status = vec![];
    for target in targets {
        let stdout = if verbosity == Verbosity::Quiet {
            std::process::Stdio::null()
        } else {
            std::process::Stdio::inherit()
        };

        if verbosity == Verbosity::Verbose {
            print!("doc-stripper");
            args.iter().for_each(|f| print!(" {}", f));
            print!(" {}", target.path.display());
            println!();
        }

        let mut command = doc_stripper_command()
            .stdout(stdout)
            .arg("-o")
            .arg(format!("{}.md", target.path.file_stem().and_then(|s| s.to_str()).unwrap()))
            .arg(target.path)
            .args(args)
            .spawn()
            .map_err(|e| match e.kind() {
                io::ErrorKind::NotFound => io::Error::new(
                    io::ErrorKind::Other,
                    "Could not run doc-stripper, please make sure it is in your PATH.",
                ),
                _ => e,
            })?;

        status.push(command.wait()?);
    }

    Ok(status
        .iter()
        .filter_map(|s| if s.success() { None } else { s.code() })
        .next()
        .unwrap_or(SUCCESS))
}

fn get_cargo_metadata(manifest_path: Option<&Path>) -> Result<cargo_metadata::Metadata, io::Error> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.no_deps();
    if let Some(manifest_path) = manifest_path {
        cmd.manifest_path(manifest_path);
    }
    cmd.other_options(vec![String::from("--offline")]);

    match cmd.exec() {
        Ok(metadata) => Ok(metadata),
        Err(_) => {
            cmd.other_options(vec![]);
            match cmd.exec() {
                Ok(metadata) => Ok(metadata),
                Err(error) => Err(io::Error::new(io::ErrorKind::Other, error.to_string())),
            }
        }
    }
}
