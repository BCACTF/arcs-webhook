DROP MATERIALIZED VIEW IF EXISTS team_data CASCADE;
CREATE MATERIALIZED VIEW team_data
AS SELECT
    id,
    array_length(solve_ids, 1) AS solve_count,

    get_solve_by_time(last_solve_time) AS last_solve,
    last_solve_time,
    COALESCE(score, 0) AS score,

    chall_ids,
    (SELECT array_agg(users.id) FROM users WHERE users.team_id = team.id) AS user_ids
FROM teams team
INNER JOIN LATERAL (
    SELECT
        MAX(solved_at) AS last_solve_time,

        SUM((SELECT points from challenges WHERE challenges.id = sol.challenge_id)) AS score,
        COALESCE(array_agg(sol.id), '{}'::uuid[]) AS solve_ids,
        COALESCE(array_agg(sol.challenge_id), '{}'::uuid[]) AS chall_ids
    FROM solve_successes sol
    WHERE sol.team_id = team.id
) sol_info
ON true;

CREATE OR REPLACE VIEW team_view
AS SELECT
    t.*,
    td.solve_count,
    td.score,
    td.last_solve,
    td.last_solve_time,
    td.user_ids,
    td.chall_ids,

    (SELECT COUNT(*) FROM solve_attempts WHERE team_id = t.id) AS attempt_count
FROM teams t
INNER JOIN team_data td
ON t.id = td.id;

CREATE UNIQUE INDEX team_data_id_idx on team_data (id);
