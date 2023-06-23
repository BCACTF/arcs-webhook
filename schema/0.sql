CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS citext;
COMMENT ON EXTENSION citext IS 'data type for case-insensitive character strings';

CREATE TABLE teams (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name citext NOT NULL UNIQUE,

    description text NOT NULL,

    score integer DEFAULT 0 NOT NULL,
    last_solve timestamp(0) without time zone,

    eligible boolean DEFAULT false NOT NULL,
    affiliation varchar(255),

    hashed_password varchar(255),
    inserted_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX teams_name_idx ON teams USING btree (name);

CREATE TABLE challenges (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name citext NOT NULL UNIQUE,

    description text NOT NULL,
    flag varchar(255) NOT NULL,
    points integer NOT NULL,

    authors varchar(255)[],
    hints varchar(255)[],
    categories text[],
    tags varchar(255)[] NOT NULL,

    solve_count integer DEFAULT 0 NOT NULL,

    visible boolean DEFAULT true NOT NULL,
    source_folder varchar(255) NOT NULL UNIQUE,
    inserted_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);
