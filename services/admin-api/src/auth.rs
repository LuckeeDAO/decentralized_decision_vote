//! Authentication and authorization for admin API

use crate::AdminError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

/// 用户角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Admin,
    Moderator,
    Viewer,
    Custom(String),
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::Admin => "admin",
            Role::Moderator => "moderator",
            Role::Viewer => "viewer",
            Role::Custom(name) => name,
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "admin" => Role::Admin,
            "moderator" => Role::Moderator,
            "viewer" => Role::Viewer,
            name => Role::Custom(name.to_string()),
        }
    }
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: Role,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub password_hash: String,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
}

/// JWT声明
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // 用户ID
    pub username: String,
    pub role: String,
    pub exp: usize,  // 过期时间
    pub iat: usize,  // 签发时间
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
}

/// 用户信息（不包含敏感信息）
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role.as_str().to_string(),
            is_active: user.is_active,
            created_at: user.created_at,
            last_login: user.last_login,
        }
    }
}

/// 创建用户请求
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub role: String,
}

/// 更新用户请求
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

/// 更改密码请求
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// 认证服务
#[derive(Clone)]
pub struct AuthService {
    jwt_secret: String,
    jwt_expiry_hours: u64,
    users: HashMap<Uuid, User>,
    username_to_id: HashMap<String, Uuid>,
}

impl AuthService {
    pub fn new(jwt_secret: String, jwt_expiry_hours: u64) -> Self {
        let mut service = Self {
            jwt_secret,
            jwt_expiry_hours,
            users: HashMap::new(),
            username_to_id: HashMap::new(),
        };
        
        // 创建默认管理员用户
        service.create_default_admin();
        service
    }

    /// 创建默认管理员用户
    fn create_default_admin(&mut self) {
        let admin_id = Uuid::new_v4();
        let admin_user = User {
            id: admin_id,
            username: "admin".to_string(),
            email: Some("admin@example.com".to_string()),
            role: Role::Admin,
            is_active: true,
            created_at: Utc::now(),
            last_login: None,
            password_hash: self.hash_password("admin123"), // 默认密码，生产环境应该更改
            failed_login_attempts: 0,
            locked_until: None,
        };
        
        self.users.insert(admin_id, admin_user);
        self.username_to_id.insert("admin".to_string(), admin_id);
    }

    /// 用户登录
    pub async fn login(&mut self, request: LoginRequest) -> Result<LoginResponse, AdminError> {
        // 查找用户
        let user_id = self.username_to_id.get(&request.username)
            .ok_or_else(|| AdminError::Authentication("Invalid username or password".to_string()))?;
        
        // 先获取用户信息进行密码验证
        let user_info = self.users.get(user_id)
            .ok_or_else(|| AdminError::Authentication("User not found".to_string()))?;

        // 检查用户是否被锁定
        if let Some(locked_until) = user_info.locked_until {
            if Utc::now() < locked_until {
                return Err(AdminError::Authentication("Account is locked".to_string()));
            }
        }

        // 验证密码
        let password_valid = self.verify_password(&request.password, &user_info.password_hash);
        
        if !password_valid {
            // 获取可变引用进行失败计数更新
            let user = self.users.get_mut(user_id).unwrap();
            user.failed_login_attempts += 1;
            
            // 检查是否需要锁定账户
            if user.failed_login_attempts >= 5 {
                user.locked_until = Some(Utc::now() + chrono::Duration::minutes(15));
            }
            
            return Err(AdminError::Authentication("Invalid username or password".to_string()));
        }

        // 获取可变引用进行成功登录更新
        let user = self.users.get_mut(user_id).unwrap();
        
        // 解锁账户（如果之前被锁定）
        if user.locked_until.is_some() {
            user.locked_until = None;
            user.failed_login_attempts = 0;
        }

        // 重置失败次数
        user.failed_login_attempts = 0;
        user.last_login = Some(Utc::now());

        // 创建用户副本用于生成令牌
        let user_for_token = user.clone();
        let jwt_secret = self.jwt_secret.clone();
        let jwt_expiry_hours = self.jwt_expiry_hours;

        // 生成JWT令牌
        let access_token = AuthService::generate_access_token_static(&user_for_token, &jwt_secret, jwt_expiry_hours)?;
        let refresh_token = AuthService::generate_refresh_token_static(&user_for_token, &jwt_secret, jwt_expiry_hours)?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: jwt_expiry_hours * 3600,
            user: UserInfo::from(user_for_token),
        })
    }

    /// 验证JWT令牌
    pub fn verify_token(&self, token: &str) -> Result<Claims, AdminError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        ).map_err(|_| AdminError::Authentication("Invalid token".to_string()))?;

        Ok(token_data.claims)
    }

    /// 创建用户
    pub async fn create_user(&mut self, request: CreateUserRequest) -> Result<UserInfo, AdminError> {
        // 检查用户名是否已存在
        if self.username_to_id.contains_key(&request.username) {
            return Err(AdminError::Validation("Username already exists".to_string()));
        }

        // 验证密码强度
        self.validate_password(&request.password)?;

        let user_id = Uuid::new_v4();
        let role = Role::from_string(&request.role);
        
        let user = User {
            id: user_id,
            username: request.username.clone(),
            email: request.email,
            role,
            is_active: true,
            created_at: Utc::now(),
            last_login: None,
            password_hash: self.hash_password(&request.password),
            failed_login_attempts: 0,
            locked_until: None,
        };

        self.users.insert(user_id, user.clone());
        self.username_to_id.insert(request.username, user_id);

        Ok(UserInfo::from(user))
    }

    /// 更新用户
    pub async fn update_user(&mut self, user_id: Uuid, request: UpdateUserRequest) -> Result<UserInfo, AdminError> {
        let user = self.users.get_mut(&user_id)
            .ok_or_else(|| AdminError::NotFound("User not found".to_string()))?;

        if let Some(username) = request.username {
            if username != user.username && self.username_to_id.contains_key(&username) {
                return Err(AdminError::Validation("Username already exists".to_string()));
            }
            
            // 更新用户名映射
            self.username_to_id.remove(&user.username);
            self.username_to_id.insert(username.clone(), user_id);
            user.username = username;
        }

        if let Some(email) = request.email {
            user.email = Some(email);
        }

        if let Some(role) = request.role {
            user.role = Role::from_string(&role);
        }

        if let Some(is_active) = request.is_active {
            user.is_active = is_active;
        }

        Ok(UserInfo::from(user.clone()))
    }

    /// 更改密码
    pub async fn change_password(&mut self, user_id: Uuid, request: ChangePasswordRequest) -> Result<(), AdminError> {
        // 先获取用户信息进行密码验证
        let user_info = self.users.get(&user_id)
            .ok_or_else(|| AdminError::NotFound("User not found".to_string()))?;

        // 验证当前密码
        let current_password_valid = self.verify_password(&request.current_password, &user_info.password_hash);
        if !current_password_valid {
            return Err(AdminError::Authentication("Current password is incorrect".to_string()));
        }

        // 验证新密码强度
        self.validate_password(&request.new_password)?;

        // 生成新密码哈希
        let jwt_secret = self.jwt_secret.clone();
        let new_password_hash = AuthService::hash_password_static(&request.new_password, &jwt_secret);

        // 获取可变引用进行密码更新
        let user = self.users.get_mut(&user_id).unwrap();
        user.password_hash = new_password_hash;

        Ok(())
    }

    /// 获取用户信息
    pub fn get_user(&self, user_id: Uuid) -> Option<UserInfo> {
        self.users.get(&user_id).map(|user| UserInfo::from(user.clone()))
    }

    /// 获取所有用户
    pub fn get_all_users(&self) -> Vec<UserInfo> {
        self.users.values().map(|user| UserInfo::from(user.clone())).collect()
    }

    /// 删除用户
    pub async fn delete_user(&mut self, user_id: Uuid) -> Result<(), AdminError> {
        if let Some(user) = self.users.remove(&user_id) {
            self.username_to_id.remove(&user.username);
            Ok(())
        } else {
            Err(AdminError::NotFound("User not found".to_string()))
        }
    }


    /// 生成访问令牌（静态方法）
    fn generate_access_token_static(user: &User, jwt_secret: &str, jwt_expiry_hours: u64) -> Result<String, AdminError> {
        let now = Utc::now().timestamp() as usize;
        let exp = now + (jwt_expiry_hours * 3600) as usize;

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: user.role.as_str().to_string(),
            exp,
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_ref()),
        ).map_err(|e| AdminError::Internal(format!("Failed to generate token: {}", e)))
    }

    /// 生成刷新令牌（静态方法）
    fn generate_refresh_token_static(user: &User, jwt_secret: &str, jwt_expiry_hours: u64) -> Result<String, AdminError> {
        // 简化实现，实际应用中应该使用更安全的刷新令牌机制
        Self::generate_access_token_static(user, jwt_secret, jwt_expiry_hours)
    }

    /// 哈希密码
    fn hash_password(&self, password: &str) -> String {
        // 简化实现，实际应用中应该使用更安全的密码哈希算法
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(self.jwt_secret.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 哈希密码（静态方法）
    fn hash_password_static(password: &str, jwt_secret: &str) -> String {
        // 简化实现，实际应用中应该使用更安全的密码哈希算法
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(jwt_secret.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 验证密码
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        self.hash_password(password) == *hash
    }

    /// 验证密码强度
    fn validate_password(&self, password: &str) -> Result<(), AdminError> {
        if password.len() < 8 {
            return Err(AdminError::Validation("Password must be at least 8 characters long".to_string()));
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        if !has_uppercase || !has_lowercase || !has_digit || !has_special {
            return Err(AdminError::Validation(
                "Password must contain at least one uppercase letter, one lowercase letter, one digit, and one special character".to_string()
            ));
        }

        Ok(())
    }
}
