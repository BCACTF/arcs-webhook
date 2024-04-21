CREATE OR REPLACE FUNCTION get_solve_by_time(solve_time timestamp(0)) RETURNS uuid AS $$
    SELECT id FROM solve_successes WHERE solved_at = $1 LIMIT 1;
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION solve_chall(solve_id uuid) RETURNS uuid as $$
    SELECT challenge_id FROM solve_successes WHERE id = $1;
$$ LANGUAGE SQL;
CREATE OR REPLACE FUNCTION solve_user(solve_id uuid) RETURNS uuid as $$
    SELECT user_id FROM solve_successes WHERE id = $1;
$$ LANGUAGE SQL;
CREATE OR REPLACE FUNCTION solve_team(solve_id uuid) RETURNS uuid as $$
    SELECT team_id FROM solve_successes WHERE id = $1;
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION chall_name(chall_id uuid) RETURNS varchar(255) as $$
    SELECT name FROM challenges WHERE id = $1;
$$ LANGUAGE SQL;
CREATE OR REPLACE FUNCTION user_name(user_id uuid) RETURNS varchar(255) as $$
    SELECT name FROM users WHERE id = $1;
$$ LANGUAGE SQL;
CREATE OR REPLACE FUNCTION team_name(team_id uuid) RETURNS varchar(255) as $$
    SELECT name FROM teams WHERE id = $1;
$$ LANGUAGE SQL;
