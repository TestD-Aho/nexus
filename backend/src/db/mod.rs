//! Database module - Connection pool and migrations

use sqlx::{postgres::{PgPoolOptions, PgRow}, Row, Executor, Pool, Postgres};
use std::time::Duration;

/// Create PostgreSQL connection pool
pub async fn create_pool(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}

/// Run database migrations
pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    // Create migrations table
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS _nexus_migrations (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL UNIQUE,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#
    ).await?;

    // Migration: Create core tables
    let migration_name = "001_create_core_tables";
    
    let already_applied: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM _nexus_migrations WHERE name = $1)"
    )
    .bind(migration_name)
    .fetch_one(pool)
    .await?;

    if !already_applied.0 {
        tracing::info!("Running migration: {}", migration_name);
        
        // Roles table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS roles (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL UNIQUE,
            description TEXT,
            is_system BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )"#).await?;

        // Permissions table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS permissions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            action VARCHAR(100) NOT NULL,
            resource VARCHAR(100) NOT NULL,
            description TEXT,
            UNIQUE(action, resource)
        )"#).await?;

        // Role-Permissions junction table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS role_permissions (
            role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
            permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
            PRIMARY KEY (role_id, permission_id)
        )"#).await?;

        // Users table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email VARCHAR(255) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            role_id UUID REFERENCES roles(id),
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            last_login TIMESTAMP WITH TIME ZONE,
            is_active BOOLEAN DEFAULT TRUE
        )"#).await?;

        // Pages table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS pages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            slug VARCHAR(255) NOT NULL UNIQUE,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            is_published BOOLEAN DEFAULT FALSE,
            is_home BOOLEAN DEFAULT FALSE,
            meta_title VARCHAR(255),
            meta_description TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            published_at TIMESTAMP WITH TIME ZONE
        )"#).await?;

        // Blocks table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS blocks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            page_id UUID REFERENCES pages(id) ON DELETE CASCADE,
            block_type VARCHAR(50) NOT NULL,
            order_index INTEGER NOT NULL DEFAULT 0,
            status VARCHAR(20) NOT NULL DEFAULT 'draft',
            title VARCHAR(255),
            content JSONB NOT NULL DEFAULT '{}',
            styling JSONB NOT NULL DEFAULT '{}',
            access_control JSONB NOT NULL DEFAULT '{"require_auth": false, "allowed_roles": ["*"]}',
            schedule_start TIMESTAMP WITH TIME ZONE,
            schedule_end TIMESTAMP WITH TIME ZONE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )"#).await?;

        // Collections table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS collections (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL UNIQUE,
            slug VARCHAR(100) NOT NULL UNIQUE,
            description TEXT,
            schema JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )"#).await?;

        // Collection items table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS collection_items (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            collection_id UUID REFERENCES collections(id) ON DELETE CASCADE,
            data JSONB NOT NULL DEFAULT '{}',
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )"#).await?;

        // Media table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS media (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            filename VARCHAR(255) NOT NULL,
            original_filename VARCHAR(255) NOT NULL,
            mime_type VARCHAR(100) NOT NULL,
            size BIGINT NOT NULL,
            url VARCHAR(500) NOT NULL,
            thumbnail_url VARCHAR(500),
            alt_text VARCHAR(255),
            uploaded_by UUID REFERENCES users(id),
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )"#).await?;

        // Projects table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS projects (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(255) NOT NULL,
            slug VARCHAR(255) NOT NULL UNIQUE,
            description TEXT,
            challenge TEXT,
            solution TEXT,
            stack JSONB,
            role VARCHAR(255),
            live_url VARCHAR(500),
            repo_url VARCHAR(500),
            media_ids JSONB,
            technologies JSONB,
            featured BOOLEAN DEFAULT FALSE,
            published BOOLEAN DEFAULT TRUE,
            published_at TIMESTAMP WITH TIME ZONE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )"#).await?;

        // Feature flags table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS feature_flags (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            key VARCHAR(100) NOT NULL UNIQUE,
            enabled BOOLEAN DEFAULT TRUE,
            description TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )"#).await?;

        // System settings table
        pool.execute(r#"CREATE TABLE IF NOT EXISTS system_settings (
            id INTEGER PRIMARY KEY DEFAULT 1,
            maintenance_mode BOOLEAN DEFAULT FALSE,
            maintenance_message TEXT,
            cv_url VARCHAR(500),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )"#).await?;

        // Insert default system settings
        pool.execute(r#"INSERT INTO system_settings (id, maintenance_mode) VALUES (1, FALSE) ON CONFLICT (id) DO NOTHING"#).await?;

        // Create indexes
        pool.execute("CREATE INDEX IF NOT EXISTS idx_blocks_page_id ON blocks(page_id)").await?;
        pool.execute("CREATE INDEX IF NOT EXISTS idx_blocks_status ON blocks(status)").await?;
        pool.execute("CREATE INDEX IF NOT EXISTS idx_collection_items_collection_id ON collection_items(collection_id)").await?;
        pool.execute("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)").await?;
        pool.execute("CREATE INDEX IF NOT EXISTS idx_pages_slug ON pages(slug)").await?;

        // Insert default roles
        pool.execute(r#"INSERT INTO roles (id, name, description, is_system) VALUES 
            ('00000000-0000-0000-0000-000000000001', 'Super-Architecte', 'Root admin - full system control', TRUE),
            ('00000000-0000-0000-0000-000000000002', 'Gestionnaire de Contenu', 'Can manage content but not system', TRUE),
            ('00000000-0000-0000-0000-000000000003', 'VIP', 'Premium authenticated user', TRUE),
            ('00000000-0000-0000-0000-000000000004', 'Visiteur', 'Anonymous visitor', TRUE)
            ON CONFLICT (name) DO NOTHING"#).await?;

        // Insert default permissions
        pool.execute(r#"INSERT INTO permissions (action, resource, description) VALUES 
            ('read', 'pages', 'Read pages'),
            ('create', 'pages', 'Create pages'),
            ('update', 'pages', 'Update pages'),
            ('delete', 'pages', 'Delete pages'),
            ('read', 'blocks', 'Read blocks'),
            ('create', 'blocks', 'Create blocks'),
            ('update', 'blocks', 'Update blocks'),
            ('delete', 'blocks', 'Delete blocks'),
            ('reorder', 'blocks', 'Reorder blocks'),
            ('read', 'collections', 'Read collections'),
            ('create', 'collections', 'Create collections'),
            ('update', 'collections', 'Update collections'),
            ('delete', 'collections', 'Delete collections'),
            ('read', 'media', 'Read media'),
            ('upload', 'media', 'Upload media'),
            ('delete', 'media', 'Delete media'),
            ('manage', 'users', 'Manage users'),
            ('manage', 'roles', 'Manage roles'),
            ('manage', 'permissions', 'Manage permissions'),
            ('toggle', 'maintenance', 'Toggle maintenance mode'),
            ('manage', 'feature-flags', 'Manage feature flags'),
            ('read', 'admin', 'Access admin dashboard'),
            ('read', 'nda', 'Read NDA-protected content'),
            ('read', 'private', 'Read private content'),
            ('read', 'public', 'Read public content')
            ON CONFLICT (action, resource) DO NOTHING"#).await?;

        // Grant all permissions to Super-Architecte
        pool.execute(r#"INSERT INTO role_permissions (role_id, permission_id)
            SELECT '00000000-0000-0000-0000-000000000001', id FROM permissions
            ON CONFLICT DO NOTHING"#).await?;

        // Grant content permissions to Gestionnaire
        pool.execute(r#"INSERT INTO role_permissions (role_id, permission_id)
            SELECT '00000000-0000-0000-0000-000000000002', id FROM permissions 
            WHERE resource IN ('pages', 'blocks', 'collections', 'media')
            AND action IN ('read', 'create', 'update', 'delete')
            ON CONFLICT DO NOTHING"#).await?;

        // Grant basic read to VIP
        pool.execute(r#"INSERT INTO role_permissions (role_id, permission_id)
            SELECT '00000000-0000-0000-0000-000000000003', id FROM permissions 
            WHERE resource IN ('pages', 'blocks')
            AND action = 'read'
            ON CONFLICT DO NOTHING"#).await?;

        // Grant public read to Visiteur
        pool.execute(r#"INSERT INTO role_permissions (role_id, permission_id)
            SELECT '00000000-0000-0000-0000-000000000004', id FROM permissions 
            WHERE resource IN ('pages', 'blocks')
            AND action = 'read'
            ON CONFLICT DO NOTHING"#).await?;

        // Record migration
        sqlx::query("INSERT INTO _nexus_migrations (name) VALUES ($1)")
            .bind(migration_name)
            .execute(pool)
            .await?;

        tracing::info!("✅ Migration {} completed", migration_name);
    }

    Ok(())
}
