use crate::{Action, Grants, ImpliedRoles, Permits, RoleSet};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct User {
    id: u32,
    roles: Vec<Role>,
    permissions: Vec<Permission>,
}

impl User {
    pub fn with_role(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }

    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.permissions.push(permission);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Editor,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostAction {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    PostRead,
    PostWrite,
}

impl ImpliedRoles for Role {
    fn parents(&self) -> impl Iterator<Item = &Self> {
        match self {
            Role::Admin => [Role::Editor].iter(),
            Role::Editor => [Role::User].iter(),
            Role::User => [].iter(),
        }
    }
}

impl RoleSet for User {
    type Role = Role;

    fn roles(&self) -> impl Iterator<Item = &Self::Role> {
        self.roles.iter()
    }
}

impl Action for PostAction {
    type Permission = Permission;

    fn required_permission(&self) -> Self::Permission {
        match self {
            PostAction::Read => Permission::PostRead,
            PostAction::Write => Permission::PostWrite,
        }
    }
}

impl Grants<Permission> for Role {
    fn grants(&self, permission: &Permission) -> bool {
        matches!(
            (self, permission),
            (Role::Editor, Permission::PostWrite) | (Role::User, Permission::PostRead)
        )
    }
}

impl Permits<Permission> for User {
    fn permits(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
}
