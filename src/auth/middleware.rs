use super::rbac::Role;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub email: String,
    pub role: Role,
    pub org_id: Option<Uuid>,
    pub is_authenticated: bool,
}

impl AuthContext {
    pub fn anonymous() -> Self {
        Self {
            user_id: Uuid::nil(),
            email: String::new(),
            role: Role::Viewer,
            org_id: None,
            is_authenticated: false,
        }
    }

    pub fn new(user_id: Uuid, email: String, role: Role, org_id: Option<Uuid>) -> Self {
        Self {
            user_id,
            email,
            role,
            org_id,
            is_authenticated: true,
        }
    }

    pub fn has_permission(&self, permission: &super::rbac::Permission) -> bool {
        super::rbac::Permission::has_permission(self.role, permission)
    }
}
