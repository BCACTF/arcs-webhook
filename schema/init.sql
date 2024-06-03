DO
$do$
BEGIN
   IF EXISTS (
      SELECT FROM pg_catalog.pg_roles
      WHERE rolname = 'arcs') THEN

      RAISE NOTICE 'Role "arcs" already exists. Skipping.';
   ELSE
      CREATE ROLE arcs;
   END IF;
END
$do$;

GRANT ALL ON DATABASE arcs TO arcs;
GRANT ALL ON SCHEMA public TO arcs;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS citext;
COMMENT ON EXTENSION citext IS 'data type for case-insensitive character strings';
