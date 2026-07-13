use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoManifest {
    pub path: PathBuf,
    pub package_name: Option<String>,
    pub dependencies: Vec<String>,
    pub has_soroban_sdk: bool,
    pub is_workspace: bool,
    pub members: Vec<PathBuf>,
}

impl CargoManifest {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            package_name: None,
            dependencies: Vec::new(),
            has_soroban_sdk: false,
            is_workspace: false,
            members: Vec::new(),
        }
    }
}
