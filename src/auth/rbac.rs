use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Owner,
    Admin,
    Developer,
    Viewer,
}

impl Role {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "owner" => Some(Self::Owner),
            "admin" => Some(Self::Admin),
            "developer" => Some(Self::Developer),
            "viewer" => Some(Self::Viewer),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Developer => "developer",
            Self::Viewer => "viewer",
        }
    }

    pub fn can_administer(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_write(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Developer)
    }

    pub fn can_read(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    CreateProject,
    UpdateProject,
    DeleteProject,
    TriggerScan,
    ViewScan,
    ViewReport,
    ManageMembers,
    ManageApiKeys,
    ManageWebhooks,
    ManageSettings,
    ViewFindings,
    Administer,
}

impl Permission {
    pub fn requires_role(role: Role) -> Vec<Self> {
        match role {
            Role::Owner => vec![
                Self::CreateProject, Self::UpdateProject, Self::DeleteProject,
                Self::TriggerScan, Self::ViewScan, Self::ViewReport,
                Self::ManageMembers, Self::ManageApiKeys, Self::ManageWebhooks,
                Self::ManageSettings, Self::ViewFindings, Self::Administer,
            ],
            Role::Admin => vec![
                Self::CreateProject, Self::UpdateProject, Self::DeleteProject,
                Self::TriggerScan, Self::ViewScan, Self::ViewReport,
                Self::ManageMembers, Self::ManageApiKeys, Self::ViewFindings,
            ],
            Role::Developer => vec![
                Self::CreateProject, Self::UpdateProject,
                Self::TriggerScan, Self::ViewScan, Self::ViewReport,
                Self::ViewFindings,
            ],
            Role::Viewer => vec![
                Self::ViewScan, Self::ViewReport, Self::ViewFindings,
            ],
        }
    }

    pub fn has_permission(role: Role, permission: &Self) -> bool {
        Self::requires_role(role).contains(permission)
    }
}
