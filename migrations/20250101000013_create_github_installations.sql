CREATE TABLE IF NOT EXISTS github_installations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    installation_id BIGINT NOT NULL,
    account_login TEXT NOT NULL,
    account_type TEXT NOT NULL DEFAULT 'Organization',
    avatar_url TEXT,
    permissions JSONB NOT NULL DEFAULT '{}',
    repository_selection TEXT NOT NULL DEFAULT 'selected',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(installation_id)
);

CREATE INDEX IF NOT EXISTS idx_github_installations_org ON github_installations(organization_id);
CREATE INDEX IF NOT EXISTS idx_github_installations_installation ON github_installations(installation_id);
