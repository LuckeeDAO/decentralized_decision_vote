//! Permission management for admin API

use crate::{AdminError, AdminOperation};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::info;

/// 权限
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    ViewSession,
    CreateSession,
    DeleteSession,
    ViewUser,
    CreateUser,
    UpdateUser,
    DeleteUser,
    ViewSystemStatus,
    ManageConfig,
    ViewLogs,
    ManagePermissions,
    ViewStatistics,
    Custom(String),
}

impl Permission {
    pub fn as_str(&self) -> &str {
        match self {
            Permission::ViewSession => "view_session",
            Permission::CreateSession => "create_session",
            Permission::DeleteSession => "delete_session",
            Permission::ViewUser => "view_user",
            Permission::CreateUser => "create_user",
            Permission::UpdateUser => "update_user",
            Permission::DeleteUser => "delete_user",
            Permission::ViewSystemStatus => "view_system_status",
            Permission::ManageConfig => "manage_config",
            Permission::ViewLogs => "view_logs",
            Permission::ManagePermissions => "manage_permissions",
            Permission::ViewStatistics => "view_statistics",
            Permission::Custom(name) => name,
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "view_session" => Permission::ViewSession,
            "create_session" => Permission::CreateSession,
            "delete_session" => Permission::DeleteSession,
            "view_user" => Permission::ViewUser,
            "create_user" => Permission::CreateUser,
            "update_user" => Permission::UpdateUser,
            "delete_user" => Permission::DeleteUser,
            "view_system_status" => Permission::ViewSystemStatus,
            "manage_config" => Permission::ManageConfig,
            "view_logs" => Permission::ViewLogs,
            "manage_permissions" => Permission::ManagePermissions,
            "view_statistics" => Permission::ViewStatistics,
            name => Permission::Custom(name.to_string()),
        }
    }

    pub fn from_operation(operation: &AdminOperation) -> Self {
        match operation {
            AdminOperation::ViewSession => Permission::ViewSession,
            AdminOperation::CreateSession => Permission::CreateSession,
            AdminOperation::DeleteSession => Permission::DeleteSession,
            AdminOperation::ViewUser => Permission::ViewUser,
            AdminOperation::CreateUser => Permission::CreateUser,
            AdminOperation::UpdateUser => Permission::UpdateUser,
            AdminOperation::DeleteUser => Permission::DeleteUser,
            AdminOperation::ViewSystemStatus => Permission::ViewSystemStatus,
            AdminOperation::ManageConfig => Permission::ManageConfig,
            AdminOperation::ViewLogs => Permission::ViewLogs,
            AdminOperation::ManagePermissions => Permission::ManagePermissions,
            AdminOperation::ViewStatistics => Permission::ViewStatistics,
        }
    }
}

/// 角色权限映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePermissions {
    pub role: String,
    pub permissions: HashSet<Permission>,
    pub inherited_roles: Vec<String>,
}

impl RolePermissions {
    pub fn new(role: String) -> Self {
        Self {
            role,
            permissions: HashSet::new(),
            inherited_roles: Vec::new(),
        }
    }

    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn add_inherited_role(&mut self, role: String) {
        if !self.inherited_roles.contains(&role) {
            self.inherited_roles.push(role);
        }
    }

    pub fn remove_inherited_role(&mut self, role: &str) {
        self.inherited_roles.retain(|r| r != role);
    }
}

/// 权限管理器
pub struct PermissionManager {
    role_permissions: HashMap<String, RolePermissions>,
    user_roles: HashMap<String, Vec<String>>, // username -> roles
    cache: HashMap<String, HashSet<Permission>>, // username -> effective permissions
    cache_ttl: u64,
    last_cache_update: std::time::Instant,
}

impl PermissionManager {
    pub fn new() -> Self {
        let mut manager = Self {
            role_permissions: HashMap::new(),
            user_roles: HashMap::new(),
            cache: HashMap::new(),
            cache_ttl: 300, // 5 minutes
            last_cache_update: std::time::Instant::now(),
        };
        
        // 初始化默认角色权限
        manager.initialize_default_roles();
        manager
    }

    /// 初始化默认角色权限
    fn initialize_default_roles(&mut self) {
        // 管理员角色
        let mut admin_role = RolePermissions::new("admin".to_string());
        admin_role.add_permission(Permission::ViewSession);
        admin_role.add_permission(Permission::CreateSession);
        admin_role.add_permission(Permission::DeleteSession);
        admin_role.add_permission(Permission::ViewUser);
        admin_role.add_permission(Permission::CreateUser);
        admin_role.add_permission(Permission::UpdateUser);
        admin_role.add_permission(Permission::DeleteUser);
        admin_role.add_permission(Permission::ViewSystemStatus);
        admin_role.add_permission(Permission::ManageConfig);
        admin_role.add_permission(Permission::ViewLogs);
        admin_role.add_permission(Permission::ManagePermissions);
        admin_role.add_permission(Permission::ViewStatistics);
        self.role_permissions.insert("admin".to_string(), admin_role);

        // 版主角色
        let mut moderator_role = RolePermissions::new("moderator".to_string());
        moderator_role.add_permission(Permission::ViewSession);
        moderator_role.add_permission(Permission::ViewUser);
        moderator_role.add_permission(Permission::ViewSystemStatus);
        moderator_role.add_permission(Permission::ViewLogs);
        moderator_role.add_permission(Permission::ViewStatistics);
        self.role_permissions.insert("moderator".to_string(), moderator_role);

        // 查看者角色
        let mut viewer_role = RolePermissions::new("viewer".to_string());
        viewer_role.add_permission(Permission::ViewSession);
        viewer_role.add_permission(Permission::ViewSystemStatus);
        viewer_role.add_permission(Permission::ViewStatistics);
        self.role_permissions.insert("viewer".to_string(), viewer_role);
    }

    /// 检查用户是否有权限执行操作
    pub fn check_permission(&mut self, username: &str, operation: &AdminOperation) -> Result<bool, AdminError> {
        let permission = Permission::from_operation(operation);
        self.has_permission(username, &permission)
    }

    /// 检查用户是否有特定权限
    pub fn has_permission(&mut self, username: &str, permission: &Permission) -> Result<bool, AdminError> {
        // 检查缓存是否过期
        if self.is_cache_expired() {
            self.refresh_cache();
        }

        // 从缓存获取权限
        if let Some(permissions) = self.cache.get(username) {
            return Ok(permissions.contains(permission));
        }

        // 计算用户的有效权限
        let effective_permissions = self.calculate_effective_permissions(username)?;
        self.cache.insert(username.to_string(), effective_permissions.clone());

        Ok(effective_permissions.contains(permission))
    }

    /// 为用户分配角色
    pub fn assign_role(&mut self, username: &str, role: String) -> Result<(), AdminError> {
        if !self.role_permissions.contains_key(&role) {
            return Err(AdminError::Validation(format!("Role '{}' does not exist", role)));
        }

        let user_roles = self.user_roles.entry(username.to_string()).or_default();
        if !user_roles.contains(&role) {
            user_roles.push(role.clone());
            info!("Assigned role to user {}: {}", username, role);
        }

        // 清除缓存
        self.cache.remove(username);
        Ok(())
    }

    /// 移除用户角色
    pub fn remove_role(&mut self, username: &str, role: &str) -> Result<(), AdminError> {
        if let Some(user_roles) = self.user_roles.get_mut(username) {
            user_roles.retain(|r| r != role);
            info!("Removed role from user {}: {}", username, role);
        }

        // 清除缓存
        self.cache.remove(username);
        Ok(())
    }

    /// 获取用户的所有角色
    pub fn get_user_roles(&self, username: &str) -> Vec<String> {
        self.user_roles.get(username).cloned().unwrap_or_default()
    }

    /// 创建新角色
    pub fn create_role(&mut self, role: String, permissions: Vec<Permission>) -> Result<(), AdminError> {
        if self.role_permissions.contains_key(&role) {
            return Err(AdminError::Validation(format!("Role '{}' already exists", role)));
        }

        let mut role_permissions = RolePermissions::new(role.clone());
        for permission in permissions {
            role_permissions.add_permission(permission);
        }

        self.role_permissions.insert(role.clone(), role_permissions);
        info!("Created new role: {}", role);
        Ok(())
    }

    /// 更新角色权限
    pub fn update_role_permissions(&mut self, role: &str, permissions: Vec<Permission>) -> Result<(), AdminError> {
        if let Some(role_permissions) = self.role_permissions.get_mut(role) {
            role_permissions.permissions.clear();
            for permission in permissions {
                role_permissions.add_permission(permission);
            }
            info!("Updated permissions for role: {}", role);
        } else {
            return Err(AdminError::NotFound(format!("Role '{}' not found", role)));
        }

        // 清除所有缓存
        self.cache.clear();
        Ok(())
    }

    /// 删除角色
    pub fn delete_role(&mut self, role: &str) -> Result<(), AdminError> {
        if !self.role_permissions.contains_key(role) {
            return Err(AdminError::NotFound(format!("Role '{}' not found", role)));
        }

        // 检查是否有用户使用此角色
        for (username, user_roles) in &self.user_roles {
            if user_roles.contains(&role.to_string()) {
                return Err(AdminError::Validation(format!(
                    "Cannot delete role '{}' because user '{}' is assigned to it",
                    role, username
                )));
            }
        }

        self.role_permissions.remove(role);
        info!("Deleted role: {}", role);

        // 清除所有缓存
        self.cache.clear();
        Ok(())
    }

    /// 获取所有角色
    pub fn get_all_roles(&self) -> Vec<String> {
        self.role_permissions.keys().cloned().collect()
    }

    /// 获取角色的权限
    pub fn get_role_permissions(&self, role: &str) -> Option<Vec<Permission>> {
        self.role_permissions.get(role).map(|rp| rp.permissions.iter().cloned().collect())
    }

    /// 计算用户的有效权限
    fn calculate_effective_permissions(&self, username: &str) -> Result<HashSet<Permission>, AdminError> {
        let mut effective_permissions = HashSet::new();
        
        if let Some(user_roles) = self.user_roles.get(username) {
            for role in user_roles {
                if let Some(role_permissions) = self.role_permissions.get(role) {
                    // 添加角色权限
                    for permission in &role_permissions.permissions {
                        effective_permissions.insert(permission.clone());
                    }
                    
                    // 添加继承的权限
                    for inherited_role in &role_permissions.inherited_roles {
                        if let Some(inherited_permissions) = self.role_permissions.get(inherited_role) {
                            for permission in &inherited_permissions.permissions {
                                effective_permissions.insert(permission.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(effective_permissions)
    }

    /// 检查缓存是否过期
    fn is_cache_expired(&self) -> bool {
        self.last_cache_update.elapsed().as_secs() > self.cache_ttl
    }

    /// 刷新缓存
    fn refresh_cache(&mut self) {
        self.cache.clear();
        self.last_cache_update = std::time::Instant::now();
        info!("Permission cache refreshed");
    }

    /// 清除用户缓存
    pub fn clear_user_cache(&mut self, username: &str) {
        self.cache.remove(username);
    }

    /// 清除所有缓存
    pub fn clear_all_cache(&mut self) {
        self.cache.clear();
        self.last_cache_update = std::time::Instant::now();
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}
