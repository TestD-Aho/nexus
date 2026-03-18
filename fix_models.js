const fs = require('fs');

let content = fs.readFileSync('backend/src/models/mod.rs', 'utf8');

const projectCode = `/// Project model
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
`;

// Remove incorrect insertion using regex
content = content.replace(/\/\/\/ Project model[\s\S]*?pub updated_at: DateTime<Utc>,\n}\n/, "");

// Insert correctly before /// Media item
content = content.replace("/// Media item", projectCode + "\n/// Media item");

fs.writeFileSync('backend/src/models/mod.rs', content);
