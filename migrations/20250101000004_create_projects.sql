CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT,
    repository_url TEXT,
    default_branch TEXT NOT NULL DEFAULT 'main',
    language TEXT NOT NULL DEFAULT 'rust',
    local_path TEXT,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(organization_id, slug)
);

CREATE INDEX IF NOT EXISTS idx_projects_org ON projects(organization_id);
