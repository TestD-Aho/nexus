const fs = require('fs');

let content = fs.readFileSync('backend/src/api/projects.rs', 'utf8');

// Fix 1: remove unused query_builder
content = content.replace(/let mut query_builder = sqlx::query_as.*?\}[\s]*\/\/ For simplicity, let's do it this way:/s, "// For simplicity, let's do it this way:");

// Fix 2: remove Option<Project> type annotation
content = content.replace(/let project: Option<Project> = sqlx::query_as::<_, Project>/g, "let project = sqlx::query_as::<_, Project>");

// Fix 3: update bind calls in update_project
content = content.replace(/\/\/ Build params \(simplified - in practice would need proper binding\)\n\s*sqlx::query\(&query\)\n\s*\.bind\(id\)\n\s*\.execute\(&state\.db_pool\)/,
`// Since dynamic queries with sqlx are hard, we'll just do a basic dynamic update
        let mut q = sqlx::query(&query).bind(id);
        
        if let Some(v) = &payload.title { q = q.bind(v); }
        if let Some(v) = &payload.slug { q = q.bind(v); }
        if let Some(v) = &payload.description { q = q.bind(v); }
        if let Some(v) = &payload.challenge { q = q.bind(v); }
        if let Some(v) = &payload.solution { q = q.bind(v); }
        if let Some(v) = &payload.stack { q = q.bind(v); }
        if let Some(v) = &payload.role { q = q.bind(v); }
        if let Some(v) = &payload.live_url { q = q.bind(v); }
        if let Some(v) = &payload.repo_url { q = q.bind(v); }
        if let Some(v) = &payload.media_ids { q = q.bind(v); }
        if let Some(v) = &payload.technologies { q = q.bind(v); }
        if let Some(v) = &payload.featured { q = q.bind(v); }
        if let Some(v) = &payload.published_at { q = q.bind(v); }
        
        q.execute(&state.db_pool)`);

fs.writeFileSync('backend/src/api/projects.rs', content);
