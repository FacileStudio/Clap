CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE spaces (
	id         text PRIMARY KEY,
	name       text NOT NULL,
	updated_at timestamptz NOT NULL DEFAULT now(),
	deleted_at timestamptz
);

CREATE TABLE space_members (
	space_id text NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
	subject  text NOT NULL,
	email    text NOT NULL,
	name     text NOT NULL DEFAULT '',
	picture  text NOT NULL DEFAULT '',
	role     text NOT NULL CHECK (role IN ('member', 'admin', 'owner')),
	PRIMARY KEY (space_id, subject)
);
CREATE INDEX space_members_subject_idx ON space_members (subject);

CREATE TABLE directory_state (
	id      smallint PRIMARY KEY CHECK (id = 1),
	version bigint NOT NULL
);

CREATE TABLE deletion_state (
	id         smallint PRIMARY KEY CHECK (id = 1),
	state      bytea NOT NULL,
	updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE projects (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	space_id   text REFERENCES spaces(id) ON DELETE CASCADE,
	owner_id   bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	title      text NOT NULL DEFAULT '',
	source     text NOT NULL,
	rev        bigint NOT NULL DEFAULT 1,
	created_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	updated_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at timestamptz NOT NULL DEFAULT now(),
	updated_at timestamptz NOT NULL DEFAULT now(),
	trashed_at timestamptz,
	CHECK (space_id IS NOT NULL OR owner_id IS NOT NULL)
);
CREATE INDEX projects_space_idx ON projects (space_id, updated_at DESC) WHERE trashed_at IS NULL;
CREATE INDEX projects_owner_idx ON projects (owner_id, updated_at DESC) WHERE space_id IS NULL AND trashed_at IS NULL;
CREATE INDEX projects_trash_idx ON projects (trashed_at) WHERE trashed_at IS NOT NULL;

CREATE TABLE project_revisions (
	id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	project_id uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	rev        bigint NOT NULL,
	source     text NOT NULL,
	created_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	origin     text NOT NULL CHECK (origin IN ('web', 'mcp', 'cli')),
	created_at timestamptz NOT NULL DEFAULT now(),
	UNIQUE (project_id, rev)
);

CREATE TABLE assets (
	id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	project_id  uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	name        text NOT NULL,
	mime        text NOT NULL,
	size        bigint NOT NULL,
	sha256      text NOT NULL,
	kind        text NOT NULL CHECK (kind IN ('video', 'audio', 'image', 'other')),
	duration_ms bigint,
	width       integer,
	height      integer,
	fps         double precision,
	has_audio   boolean NOT NULL DEFAULT false,
	poster      text,
	proxy       text,
	state       text NOT NULL DEFAULT 'probing' CHECK (state IN ('probing', 'ready', 'failed')),
	error       text,
	created_by  bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	created_at  timestamptz NOT NULL DEFAULT now(),
	UNIQUE (project_id, name)
);
CREATE INDEX assets_sha256_idx ON assets (sha256);
CREATE INDEX assets_probing_idx ON assets (created_at) WHERE state = 'probing';

CREATE TABLE renders (
	id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	project_id   uuid NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
	requested_by bigint REFERENCES porte_users(id) ON DELETE SET NULL,
	rev          bigint NOT NULL,
	width        integer NOT NULL,
	height       integer NOT NULL,
	fps          integer NOT NULL,
	range_start  double precision,
	range_end    double precision,
	state        text NOT NULL DEFAULT 'queued'
		CHECK (state IN ('queued', 'running', 'done', 'failed', 'cancelled')),
	frame        bigint NOT NULL DEFAULT 0,
	frames       bigint NOT NULL DEFAULT 0,
	error        text,
	output       text,
	size         bigint,
	created_at   timestamptz NOT NULL DEFAULT now(),
	started_at   timestamptz,
	finished_at  timestamptz,
	CHECK ((range_start IS NULL) = (range_end IS NULL))
);
CREATE INDEX renders_project_idx ON renders (project_id, created_at DESC);
CREATE INDEX renders_open_idx ON renders (created_at) WHERE state IN ('queued', 'running');
