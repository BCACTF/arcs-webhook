CREATE TABLE IF NOT EXISTS teams (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name citext NOT NULL UNIQUE,

    description text NOT NULL,

    eligible boolean DEFAULT false NOT NULL,
    affiliation varchar(255),

    hashed_password varchar(255) NOT NULL,
    inserted_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS teams_name_idx ON teams USING btree (name);

CREATE TABLE IF NOT EXISTS challenges (
    id uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name citext NOT NULL UNIQUE,

    description text NOT NULL,
    flag varchar(255) NOT NULL,
    points integer NOT NULL,

    authors varchar(255)[] NOT NULL,
    hints varchar(255)[] NOT NULL,
    categories text[] NOT NULL,
    tags varchar(255)[] NOT NULL,

    solve_count integer DEFAULT 0 NOT NULL,

    visible boolean DEFAULT true NOT NULL,
    source_folder varchar(255) NOT NULL UNIQUE,
    inserted_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp(0) without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);
