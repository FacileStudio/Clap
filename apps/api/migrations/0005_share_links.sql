CREATE TABLE share_links (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	token_hash text NOT NULL UNIQUE,
	created_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at timestamptz NOT NULL DEFAULT now(),
	revoked_at timestamptz
);
CREATE UNIQUE INDEX share_links_live_idx ON share_links (project_id) WHERE revoked_at IS NULL;
CREATE INDEX renders_done_idx ON renders (project_id, finished_at DESC) WHERE state = 'done';
