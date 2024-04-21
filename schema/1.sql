DROP TYPE IF EXISTS link_type CASCADE;
CREATE TYPE link_type AS ENUM (
    'static',
    'web',
    'nc',
    'admin'
);
DROP TABLE IF EXISTS challenge_links;
CREATE TABLE challenge_links (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    challenge_id uuid NOT NULL,
    url text NOT NULL,
    type link_type NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS chall_id_url_idx ON challenge_links USING btree (challenge_id, url);

ALTER TABLE challenge_links DROP
    CONSTRAINT IF EXISTS fkey_cl_chalid;
ALTER TABLE ONLY challenge_links ADD
    CONSTRAINT fkey_cl_chalid FOREIGN KEY (challenge_id) REFERENCES challenges(id) ON DELETE CASCADE;


CREATE TABLE IF NOT EXISTS users (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    email citext NOT NULL UNIQUE,
    name citext NOT NULL UNIQUE,

    team_id uuid,

    eligible boolean DEFAULT false NOT NULL,
    admin boolean DEFAULT false NOT NULL,

    inserted_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    confirmed_at timestamp(0) without time zone
);
CREATE UNIQUE INDEX IF NOT EXISTS user_name_idx ON users USING btree (name);
CREATE UNIQUE INDEX IF NOT EXISTS user_email_idx ON users USING btree (email);


ALTER TABLE users DROP
    CONSTRAINT IF EXISTS fkey_u_teamid;
ALTER TABLE ONLY users ADD
    CONSTRAINT fkey_u_teamid FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE;

