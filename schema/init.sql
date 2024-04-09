CREATE ROLE arcs;
GRANT ALL ON DATABASE arcs TO arcs;
GRANT ALL ON SCHEMA public TO arcs;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS citext;
COMMENT ON EXTENSION citext IS 'data type for case-insensitive character strings';
