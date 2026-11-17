CREATE TABLE markers (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	t          double precision NOT NULL CHECK (t >= 0),
	label      text NOT NULL DEFAULT '' CHECK (length(label) <= 200),
	created_at timestamptz DEFAULT now()
);
CREATE INDEX markers_project_idx ON markers (project_id);
