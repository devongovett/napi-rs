use crate::util::{get_system_default_target, CommandResult, Executable};
use clap::Args;
use clap_cargo::{Features, Workspace};
use log::trace;
use rand::{thread_rng, RngCore};
use std::fmt::Write;
use std::{env::temp_dir, path::PathBuf, process::Command};

#[derive(Args, Debug, Default)]
/// build the napi-rs crates
pub struct BuildCommand {
  /// Build for the target triple, bypassed to `cargo build --target`
  #[clap(short, long)]
  target: Option<String>,

  /// Path to the generate JS binding file. Only work when `--target` is specified
  #[clap(long = "js", parse(from_os_str))]
  js_binding: Option<PathBuf>,

  /// Disable JS binding file generation
  #[clap(long = "no-js")]
  disable_js_binding: bool,

  /// Path to the `Cargo.toml` manifest
  #[clap(long, parse(from_os_str))]
  cwd: Option<PathBuf>,

  /// Path to where all the built files would be put
  #[clap(short, long, parse(from_os_str))]
  dest: Option<PathBuf>,

  /// Whether strip the library to achieve the minimum file size
  #[clap(short, long)]
  strip: bool,

  /// Pipe that will receive all generated js/ts files as input. Usage: `--pipe="prettier -w"`
  #[clap(long)]
  pipe: Option<String>,

  /// Build in release mode
  #[clap(short, long)]
  release: bool,

  /// Verbosely log build command trace
  #[clap(short, long)]
  verbose: bool,

  #[clap(flatten)]
  features: Features,

  #[clap(flatten)]
  workspace: Workspace,

  /// see https://github.com/napi-rs/napi-rs/issues/297
  /// Disable windows x32 lto and increase `codegen-units`.
  #[clap(long)]
  disable_windows_x32_optimize: bool,

  /// [experimental] Use `zig` as linker (cross-compile)
  #[clap(short, long)]
  zig: bool,

  /// [experimental] The suffix of zig ABI version. E.g. `--zig-abi-suffix=2.17`
  #[clap(long)]
  zip_abi_suffix: Option<String>,

  #[clap(skip)]
  intermediate_type_file: PathBuf,

  /// All other flags bypassed to `cargo build` command. Usage: `napi build -- -p sub-crate`
  #[clap(last = true)]
  bypass_flags: Vec<String>,
}

impl Executable for BuildCommand {
  fn execute(&mut self) -> CommandResult {
    if self.verbose {
      log::set_max_level(log::LevelFilter::Trace)
    }

    self.run();

    Ok(())
  }
}

impl BuildCommand {
  fn run(&mut self) {
    self.intermediate_type_file = get_intermediate_type_file();

    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    self
      .set_cwd(&mut cmd)
      .set_features(&mut cmd)
      .set_workspace(&mut cmd)
      .set_target(&mut cmd)
      .set_envs(&mut cmd)
      .set_bypass_args(&mut cmd);

    cmd.spawn().expect("failed to execute `cargo build`");
  }

  fn set_cwd(&mut self, cmd: &mut Command) -> &mut Self {
    if let Some(cwd) = &self.cwd {
      trace!("set cargo working dir to {}", cwd.display());
      cmd.current_dir(cwd);
    }

    self
  }

  fn set_envs(&mut self, cmd: &mut Command) -> &mut Self {
    let mut envs = vec![(
      "TYPE_DEF_TMP_PATH",
      self.intermediate_type_file.to_str().unwrap(),
    )];

    if self.disable_windows_x32_optimize && self.target.as_deref() == Some("i686-pc-windows-msvc") {
      envs.extend([
        ("CARGO_PROFILE_DEBUG_CODEGEN_UNITS", "256"),
        ("CARGO_PROFILE_RELEASE_CODEGEN_UNITS", "256"),
        ("CARGO_PROFILE_RELEASE_LTO", r#""off""#),
      ])
    }

    trace!("set environment variables: ");
    envs.iter().for_each(|(k, v)| {
      trace!("{}={}", k, v);
      cmd.env(k, v);
    });

    self
  }

  fn set_target(&mut self, cmd: &mut Command) -> &mut Self {
    if let Some(target) = &self.target {
      trace!("set compiling target to {}", target);
      cmd.arg("--target").arg(target);
    } else {
      self.target.replace(get_system_default_target());
    }

    self
  }

  fn set_bypass_args(&mut self, cmd: &mut Command) -> &mut Self {
    trace!("bypassing flags: {:?}", self.bypass_flags);

    cmd.args(self.bypass_flags.iter());

    self
  }

  fn set_features(&mut self, cmd: &mut Command) -> &mut Self {
    let mut args = vec![];
    if self.features.all_features {
      args.push(String::from("--all-features"))
    } else if self.features.no_default_features {
      args.push(String::from("--no-default-features"))
    } else if !self.features.features.is_empty() {
      args.push(String::from("--features"));
      args.extend_from_slice(&self.features.features);
    }

    trace!("set features flags: {:?}", args);
    cmd.args(args);

    self
  }

  fn set_workspace(&mut self, cmd: &mut Command) -> &mut Self {
    let mut args = vec![];
    if self.workspace.all || self.workspace.workspace {
      args.push(String::from("--workspace"));
    } else if !self.workspace.package.is_empty() {
      args.push(String::from("-p"));
      args.extend_from_slice(&self.workspace.package);
    }

    if !self.workspace.exclude.is_empty() {
      args.push(String::from("--exclude"));
      args.extend_from_slice(&self.workspace.exclude);
    }

    trace!("set workspace flags: {:?}", args);
    cmd.args(args);

    self
  }
}

fn get_intermediate_type_file() -> PathBuf {
  let len = 16;
  let mut rng = thread_rng();
  let mut data = vec![0; len];
  rng.fill_bytes(&mut data);

  let mut hex_string = String::with_capacity(2 * len);
  for byte in data {
    write!(hex_string, "{:02X}", byte).unwrap();
  }

  temp_dir().join(format!("type_def.{hex_string}.tmp"))
}
