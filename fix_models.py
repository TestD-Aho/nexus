import sys

with open('backend/src/models/mod.rs', 'r') as f:
    content = f.read()

# The incorrect insertion placed `/// Project model` and its struct right after `#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]` of Media.
# Let's just remove the bad Project block and insert it correctly.

project_code = """
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
"""

# Let's find the incorrect block and replace it.
import re

# Find the exact text from `/// Project model` to `pub updated_at: DateTime<Utc>,\n}`
pattern = r"/// Project model.*?pub updated_at: DateTime<Utc>,\n}\n"
content = re.sub(pattern, "", content, flags=re.DOTALL)

# Now find `/// Media item` and insert `project_code` before it.
content = content.replace("/// Media item", project_code + "\n/// Media item")

with open('backend/src/models/mod.rs', 'w') as f:
    f.write(content)

