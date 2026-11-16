ALTER TABLE assets
	ADD COLUMN strip       text,
	ADD COLUMN strip_tiles integer CHECK (strip_tiles BETWEEN 1 AND 60),
	ADD COLUMN strip_step  double precision CHECK (strip_step > 0),
	ADD COLUMN peaks       text;
