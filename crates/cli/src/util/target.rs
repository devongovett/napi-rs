use phf::phf_map;
use serde::{ser::SerializeMap, Serialize, Serializer};

pub const AVAILABLE_TARGETS: &[&str] = &[
  "aarch64-apple-darwin",
  "aarch64-linux-android",
  "aarch64-unknown-linux-gnu",
  "aarch64-unknown-linux-musl",
  "aarch64-pc-windows-msvc",
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
  "x86_64-unknown-linux-musl",
  "x86_64-unknown-freebsd",
  "i686-pc-windows-msvc",
  "armv7-unknown-linux-gnueabihf",
  "armv7-linux-androideabi",
];

pub const DEFAULT_TARGETS: &[&str] = &[
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
];

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum NodeArch {
  x32,
  x64,
  ia32,
  arm,
  arm64,
  mips,
  mipsel,
  ppc,
  ppc64,
  s390,
  s390x,
}

impl NodeArch {
  fn from_str(s: &str) -> Option<Self> {
    match s {
      "x32" => Some(NodeArch::x32),
      "x86_64" => Some(NodeArch::x64),
      "i686" => Some(NodeArch::ia32),
      "armv7" => Some(NodeArch::arm),
      "aarch64" => Some(NodeArch::arm64),
      "mips" => Some(NodeArch::mips),
      "mipsel" => Some(NodeArch::mipsel),
      "ppc" => Some(NodeArch::ppc),
      "ppc64" => Some(NodeArch::ppc64),
      "s390" => Some(NodeArch::s390),
      "s390x" => Some(NodeArch::s390x),
      _ => None,
    }
  }
}

impl std::fmt::Display for NodeArch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NodeArch::x32 => write!(f, "x86"),
      NodeArch::x64 => write!(f, "x64"),
      NodeArch::ia32 => write!(f, "ia32"),
      NodeArch::arm => write!(f, "arm"),
      NodeArch::arm64 => write!(f, "arm64"),
      NodeArch::mips => write!(f, "mips"),
      NodeArch::mipsel => write!(f, "mipsel"),
      NodeArch::ppc => write!(f, "ppc"),
      NodeArch::ppc64 => write!(f, "ppc64"),
      NodeArch::s390 => write!(f, "s390"),
      NodeArch::s390x => write!(f, "s390x"),
    }
  }
}

impl NodeArch {
  fn as_github_action_arch(&self) -> &str {
    match self {
      NodeArch::x32 => "x86",
      NodeArch::x64 => "x64",
      _ => "x64",
    }
  }
}

impl Serialize for NodeArch {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(self.as_github_action_arch())
  }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum NodePlatform {
  darwin,
  freebsd,
  openbsd,
  win32,
  unknown(String),
}

impl NodePlatform {
  fn from_str(s: &str) -> Self {
    match s {
      "darwin" => NodePlatform::darwin,
      "freebsd" => NodePlatform::freebsd,
      "openbsd" => NodePlatform::openbsd,
      "windows" => NodePlatform::win32,
      _ => NodePlatform::unknown(s.to_owned()),
    }
  }
}

impl std::fmt::Display for NodePlatform {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NodePlatform::darwin => write!(f, "darwin"),
      NodePlatform::freebsd => write!(f, "freebsd"),
      NodePlatform::openbsd => write!(f, "openbsd"),
      NodePlatform::win32 => write!(f, "win32"),
      NodePlatform::unknown(s) => write!(f, "{}", s),
    }
  }
}

impl Serialize for NodePlatform {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&format!("{}", self))
  }
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetDetail {
  pub platform_abi: String,
  pub arch: NodeArch,
  pub platform: NodePlatform,
  pub abi: Option<String>,
}

impl From<&str> for TargetDetail {
  fn from(triple: &str) -> TargetDetail {
    let parts = triple.split('-').collect::<Vec<_>>();
    let (cpu, sys, abi) = if parts.len() == 2 {
      (parts[0], parts[2], None)
    } else {
      (parts[0], parts[2], parts.get(3))
    };

    let platform = NodePlatform::from_str(sys);
    let arch = NodeArch::from_str(cpu).unwrap_or_else(|| panic!("unsupported cpu arch {}", cpu));
    TargetDetail {
      platform_abi: if abi.is_some() {
        format!("{}-{}-{}", platform, arch, abi.unwrap())
      } else {
        format!("{}-{}", platform, arch)
      },
      arch,
      platform,
      abi: abi.map(|s| s.to_string()),
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct GithubWorkflowConfig {
  pub host: &'static str,
  pub docker_image: Option<&'static str>,
  pub setup: Option<&'static str>,
}

impl Serialize for GithubWorkflowConfig {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(2))?;
    map.serialize_entry("host", self.host)?;
    if let Some(docker_image) = &self.docker_image {
      map.serialize_entry("docker_image", docker_image)?;
    }
    if let Some(setup) = &self.setup {
      let scripts = setup.split("&&").map(|s| s.trim()).collect::<Vec<_>>();
      map.serialize_entry("setup", &scripts)?;
    }
    map.end()
  }
}

static TARGET_CONFIG_MAP: phf::Map<&'static str, GithubWorkflowConfig> = phf_map! {
  "x86_64-apple-darwin" => GithubWorkflowConfig {
    host: "macos-latest",
    docker_image: None,
    setup: None,
  },
  "x86_64-pc-windows-msvc" => GithubWorkflowConfig {
    host: "windows-latest",
    docker_image: None,
    setup: None,
  },
  "i686-pc-windows-msvc" => GithubWorkflowConfig {
    host: "windows-latest",
    docker_image: None,
    setup: None,
  },
  "x86_64-unknown-linux-gnu" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: Some("napi-rs/nodejs-rust:lts-debian"),
    setup: None,
  },
  "x86_64-unknown-linux-musl" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: Some("napi-rs/nodejs-rust:lts-alpine"),
    setup: None,
  },
  // CHECK
  "x86_64-unknown-freebsd" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: None,
    setup: None,
  },
  "aarch64-apple-darwin" => GithubWorkflowConfig {
    host: "macos-latest",
    docker_image: None,
    setup: None,
  },
  "aarch64-unknown-linux-gnu" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: None,
    setup: Some("sudo apt-get update && sudo apt-get install g++-aarch64-linux-gnu gcc-aarch64-linux-gnu -y"),
  },
  "aarch64-unknown-linux-musl" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: Some("napi-rs/nodejs-rust:lts-alpine"),
    setup: None,
  },
  "aarch64-pc-windows-msvc" => GithubWorkflowConfig {
    host: "windows-latest",
    docker_image: None,
    setup: None,
  },
  "aarch64-linux-android" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: None,
    setup: None,
  },
  "armv7-unknown-linux-gnueabihf" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: None,
    setup: Some("sudo apt-get update && sudo apt-get install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf -y"),
  },
  "armv7-linux-androideabi" => GithubWorkflowConfig {
    host: "ubuntu-latest",
    docker_image: None,
    setup: None,
  },
};

#[derive(Clone, Debug, Serialize)]
pub struct Target {
  pub triple: String,
  pub detail: TargetDetail,
  pub github_workflow_config: GithubWorkflowConfig,
}

impl Target {
  pub fn new(triple: &str) -> Self {
    let detail = TargetDetail::from(triple);
    let config = TARGET_CONFIG_MAP.get(triple).unwrap().clone();
    Self {
      triple: triple.to_owned(),
      detail,
      github_workflow_config: config,
    }
  }
}
