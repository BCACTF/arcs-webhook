CREATE TABLE IF NOT EXISTS solve_attempts (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),

    flag_guess varchar(255) NOT NULL,
    correct boolean DEFAULT false NOT NULL,

    user_id uuid NOT NULL,
    challenge_id uuid NOT NULL,
    team_id uuid NOT NULL,

    inserted_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at timestamp(0) without time zone
);

CREATE OR REPLACE VIEW solve_successes AS
SELECT *, inserted_at AS solved_at FROM (
    SELECT
        *,
        ROW_NUMBER() OVER (
            PARTITION BY challenge_id, team_id ORDER BY inserted_at ASC
        ) AS row_number
    FROM solve_attempts
    WHERE correct = true AND deleted_at IS NULL
) numbered_solves
WHERE row_number = 1;

CREATE OR REPLACE VIEW solve_fails AS
SELECT *
FROM solve_attempts
WHERE correct = false AND deleted_at IS NULL;


CREATE OR REPLACE VIEW deleted_solves AS
SELECT *
FROM solve_attempts
WHERE deleted_at IS NOT NULL;


CREATE TABLE IF NOT EXiSTS auth_name_pass (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL,

    hashed_password varchar(255) NOT NULL,
    
    last_used timestamp(0) without time zone,

    inserted_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXiSTS auth_oauth (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    user_id uuid NOT NULL,

    provider_name text NULL,
    sub varchar(255) NOT NULL,

    last_used timestamp(0) without time zone,

    inserted_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);


ALTER TABLE solve_attempts DROP
    CONSTRAINT IF EXISTS fkey_u_chalid;
ALTER TABLE ONLY solve_attempts ADD
    CONSTRAINT fkey_u_chalid FOREIGN KEY (challenge_id) REFERENCES challenges(id) ON DELETE CASCADE;

ALTER TABLE solve_attempts DROP
    CONSTRAINT IF EXISTS fkey_u_teamid;
ALTER TABLE ONLY solve_attempts ADD
    CONSTRAINT fkey_u_teamid FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE;

ALTER TABLE solve_attempts DROP
    CONSTRAINT IF EXISTS fkey_u_userid;
ALTER TABLE ONLY solve_attempts ADD
    CONSTRAINT fkey_u_userid FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;


ALTER TABLE auth_name_pass DROP
    CONSTRAINT IF EXISTS fkey_user_id;
ALTER TABLE ONLY auth_name_pass ADD
    CONSTRAINT fkey_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

ALTER TABLE auth_oauth DROP
    CONSTRAINT IF EXISTS fkey_user_id;
ALTER TABLE ONLY auth_oauth ADD
    CONSTRAINT fkey_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;


CREATE INDEX IF NOT EXISTS solves_chalid_idx ON solve_attempts USING btree (challenge_id);
CREATE INDEX IF NOT EXISTS solves_teamid_idx ON solve_attempts USING btree (team_id);
CREATE INDEX IF NOT EXISTS solves_userid_idx ON solve_attempts USING btree (user_id);


CREATE UNIQUE INDEX IF NOT EXISTS auth_name_pass_user_id ON auth_name_pass USING btree (user_id);

CREATE INDEX IF NOT EXISTS auth_oauth_user_id ON auth_oauth USING btree (user_id);
CREATE INDEX IF NOT EXISTS auth_oauth_provider ON auth_oauth USING btree (provider_name);
CREATE UNIQUE INDEX IF NOT EXISTS oauth_user_id_unique ON auth_oauth USING btree (user_id, provider_name);
