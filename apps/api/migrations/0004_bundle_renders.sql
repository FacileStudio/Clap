ALTER TABLE renders ALTER COLUMN project_id DROP NOT NULL;
CREATE INDEX renders_requester_idx ON renders (requested_by, created_at DESC) WHERE project_id IS NULL;
