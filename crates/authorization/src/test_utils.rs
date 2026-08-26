use crate::{HasPermission, HasPermissionOn, HasRole, Subject};

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
    Read,
    Write,
}

impl Subject for User {
    type Identity = u32;

    fn identity(&self) -> Self::Identity {
        self.id
    }
}

impl HasRole for User {
    type Role = Role;

    fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }
}

impl HasPermission<&'static str> for User {
    type Permission = Permission;

    fn has_permission(&self, permission: &Permission, _action: &&'static str) -> bool {
        self.permissions.contains(permission)
    }
}

impl HasPermissionOn<&'static str, Post> for User {
    type Permission = Permission;

    fn has_permission_on(
        &self,
        permission: &Permission,
        _action: &&'static str,
        resource: &Post,
    ) -> bool {
        self.permissions.contains(permission)
            && match permission {
                Permission::Read => true,
                Permission::Write => resource.owner_id == self.id,
            }
    }
}
