#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArch {
    X86_32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOs {
    Nyxara,
    Linux,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub arch: TargetArch,
    pub os: TargetOs,
}

impl Target {
    pub fn nyxara_x86() -> Self {
        Self {
            arch: TargetArch::X86_32,
            os: TargetOs::Nyxara,
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "nyxara" | "nyxara-x86" | "i386-nyxara" => Ok(Self::nyxara_x86()),
            "linux" | "linux-x86" | "i386-linux" => Ok(Self {
                arch: TargetArch::X86_32,
                os: TargetOs::Linux,
            }),
            _ => Err(format!("Unknown target triple '{}'", s)),
        }
    }
}
