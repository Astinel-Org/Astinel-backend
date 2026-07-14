CREATE TABLE IF NOT EXISTS notification_events (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    event_type VARCHAR(64) NOT NULL,
    title VARCHAR(256) NOT NULL,
    message TEXT NOT NULL,
    severity VARCHAR(32) NOT NULL DEFAULT 'info',
    resource_type VARCHAR(64),
    resource_id UUID,
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notification_events_org ON notification_events(organization_id, created_at DESC);
CREATE INDEX idx_notification_events_unread ON notification_events(organization_id, is_read) WHERE is_read = false;
