CREATE TABLE IF NOT EXISTS scan_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_job_id UUID NOT NULL UNIQUE REFERENCES scan_jobs(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    total_files INTEGER NOT NULL DEFAULT 0,
    total_rules INTEGER NOT NULL DEFAULT 0,
    total_findings INTEGER NOT NULL DEFAULT 0,
    suppressed_findings INTEGER NOT NULL DEFAULT 0,
    critical INTEGER NOT NULL DEFAULT 0,
    high INTEGER NOT NULL DEFAULT 0,
    medium INTEGER NOT NULL DEFAULT 0,
    low INTEGER NOT NULL DEFAULT 0,
    info INTEGER NOT NULL DEFAULT 0,
    score INTEGER NOT NULL DEFAULT 100,
    duration_ms BIGINT NOT NULL DEFAULT 0,
    raw_output TEXT,
    report_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_scan_results_job ON scan_results(scan_job_id);
