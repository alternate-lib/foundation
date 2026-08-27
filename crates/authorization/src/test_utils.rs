use crate::{Action, HasPermission, HasPermissionOn, HasRole};

#[derive(Debug, Default)]
pub struct User {
    id: u32,
    roles: Vec<Role>,
    permissions: Vec<Permission>,
}

impl User {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            roles: Vec::new(),
            permissions: Vec::new(),
        }
    }

    pub fn with_role(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }

    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.permissions.push(permission);
        self
    }
}

#[derive(Debug, Default)]
pub struct Post {
    owner_id: u32,
}

impl Post {
    pub fn with_owner(owner_id: u32) -> Self {
        Self { owner_id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Editor,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    PostRead,
    PostWrite,
}

impl HasRole for User {
    type Role = Role;

    fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostAction {
    Read,
    Write,
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

impl HasPermission for User {
    type Permission = Permission;

    fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
}

impl HasPermissionOn<Post> for User {
    type Permission = Permission;

    fn has_permission_on(&self, permission: &Permission, resource: &Post) -> bool {
        self.permissions.contains(permission)
            && match permission {
                Permission::PostRead => true,
                Permission::PostWrite => resource.owner_id == self.id,
            }
    }
}
