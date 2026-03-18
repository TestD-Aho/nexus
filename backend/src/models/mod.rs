//! Core Data Models for Nexus

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

/// Role model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

/// Permission model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub action: String,
    pub resource: String,
    pub description: Option<String>,
}

/// Role-Permission association
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermission {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}

/// Page model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Page {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub is_published: bool,
    pub is_home: bool,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Block types enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum BlockType {
    HeroHeader,
    WorkProcess,
    RichText,
    ProjectGrid,
    SkillMatrix,
    ContactForm,
    TestimonialSlider,
    Custom,
}

impl std::fmt::Display for BlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockType::HeroHeader => write!(f, "HeroHeader"),
            BlockType::WorkProcess => write!(f, "WorkProcess"),
            BlockType::RichText => write!(f, "RichText"),
            BlockType::ProjectGrid => write!(f, "ProjectGrid"),
            BlockType::SkillMatrix => write!(f, "SkillMatrix"),
            BlockType::ContactForm => write!(f, "ContactForm"),
            BlockType::TestimonialSlider => write!(f, "TestimonialSlider"),
            BlockType::Custom => write!(f, "Custom"),
        }
    }
}

impl std::str::FromStr for BlockType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HeroHeader" => Ok(BlockType::HeroHeader),
            "WorkProcess" => Ok(BlockType::WorkProcess),
            "RichText" => Ok(BlockType::RichText),
            "ProjectGrid" => Ok(BlockType::ProjectGrid),
            "SkillMatrix" => Ok(BlockType::SkillMatrix),
            "ContactForm" => Ok(BlockType::ContactForm),
            "TestimonialSlider" => Ok(BlockType::TestimonialSlider),
            "Custom" => Ok(BlockType::Custom),
            _ => Err(format!("Unknown block type: {}", s)),
        }
    }
}

/// Block status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlockStatus {
    Draft,
    Published,
    Archived,
}

impl std::fmt::Display for BlockStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockStatus::Draft => write!(f, "draft"),
            BlockStatus::Published => write!(f, "published"),
            BlockStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for BlockStatus {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(BlockStatus::Draft),
            "published" => Ok(BlockStatus::Published),
            "archived" => Ok(BlockStatus::Archived),
            _ => Err(format!("Unknown block status: {}", s)),
        }
    }
}

/// Block model with JSON content
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Block {
    pub id: Uuid,
    pub page_id: Uuid,
    pub block_type: String,
    pub order_index: i32,
    pub status: String,
    pub title: Option<String>,
    pub content: serde_json::Value,
    pub styling: serde_json::Value,
    pub access_control: serde_json::Value,
    pub schedule_start: Option<DateTime<Utc>>,
    pub schedule_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Collection (Custom Post Type)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub schema: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Collection item
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CollectionItem {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Project model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub challenge: Option<String>,
    pub solution: Option<String>,
    pub stack: Option<serde_json::Value>,
    pub role: Option<String>,
    pub live_url: Option<String>,
    pub repo_url: Option<String>,
    pub media_ids: Option<serde_json::Value>,
    pub technologies: Option<serde_json::Value>,
    pub featured: bool,
    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Media item
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]

pub struct Media {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub size: i64,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub alt_text: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Feature flag
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FeatureFlag {
    pub id: Uuid,
    pub key: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// System settings
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemSettings {
    pub id: i32,
    pub maintenance_mode: bool,
    pub maintenance_message: Option<String>,
    pub updated_at: DateTime<Utc>,
}

// ============ Request/Response DTOs ============

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Register request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub role: Option<String>,
}

/// Auth response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

/// User response (without sensitive data)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

/// Create page request
#[derive(Debug, Deserialize)]
pub struct CreatePageRequest {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub is_published: Option<bool>,
    pub is_home: Option<bool>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

/// Update page request
#[derive(Debug, Deserialize)]
pub struct UpdatePageRequest {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_published: Option<bool>,
    pub is_home: Option<bool>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
}

/// Create block request
#[derive(Debug, Deserialize)]
pub struct CreateBlockRequest {
    pub page_id: Uuid,
    pub block_type: String,
    pub title: Option<String>,
    pub content: serde_json::Value,
    pub styling: Option<serde_json::Value>,
    pub access_control: Option<serde_json::Value>,
    pub schedule_start: Option<DateTime<Utc>>,
    pub schedule_end: Option<DateTime<Utc>>,
}

/// Update block request
#[derive(Debug, Deserialize)]
pub struct UpdateBlockRequest {
    pub block_type: Option<String>,
    pub title: Option<String>,
    pub content: Option<serde_json::Value>,
    pub styling: Option<serde_json::Value>,
    pub access_control: Option<serde_json::Value>,
    pub status: Option<String>,
    pub schedule_start: Option<DateTime<Utc>>,
    pub schedule_end: Option<DateTime<Utc>>,
}

/// Reorder blocks request
#[derive(Debug, Deserialize)]
pub struct ReorderBlocksRequest {
    pub blocks: Vec<BlockReorderItem>,
}

#[derive(Debug, Deserialize)]
pub struct BlockReorderItem {
    pub id: Uuid,
    pub order_index: i32,
}

/// Create collection request
#[derive(Debug, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub schema: serde_json::Value,
}

/// Collection item request
#[derive(Debug, Deserialize)]
pub struct CreateCollectionItemRequest {
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCollectionItemRequest {
    pub data: serde_json::Value,
}

/// Role request
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Uuid>,
}

/// Admin stats response
#[derive(Debug, Serialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub total_pages: i64,
    pub total_blocks: i64,
    pub total_collections: i64,
    pub total_media: i64,
    pub active_sessions: i64,
}

/// Create project request
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub challenge: Option<String>,
    pub solution: Option<String>,
    pub stack: Option<serde_json::Value>,
    pub role: Option<String>,
    pub live_url: Option<String>,
    pub repo_url: Option<String>,
    pub media_ids: Option<serde_json::Value>,
    pub technologies: Option<serde_json::Value>,
    pub featured: Option<bool>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Update project request
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub challenge: Option<String>,
    pub solution: Option<String>,
    pub stack: Option<serde_json::Value>,
    pub role: Option<String>,
    pub live_url: Option<String>,
    pub repo_url: Option<String>,
    pub media_ids: Option<serde_json::Value>,
    pub technologies: Option<serde_json::Value>,
    pub featured: Option<bool>,
    pub published_at: Option<DateTime<Utc>>,
}
