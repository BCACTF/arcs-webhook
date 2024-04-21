DROP MATERIALIZED VIEW IF EXISTS user_data CASCADE;
CREATE MATERIALIZED VIEW IF NOT EXISTS user_data
AS SELECT
    users.id,
    COALESCE(array_length(solve_ids, 1), 0) AS solve_count,

    get_solve_by_time(last_solve_time) AS last_solve,
    last_solve_time,
    COALESCE(score, 0) AS score,

    COALESCE(chall_ids, '{}'::uuid[]) AS chall_ids
FROM users
INNER JOIN LATERAL (
    SELECT
        MAX(solved_at) AS last_solve_time,

        SUM((SELECT points from challenges WHERE challenges.id = sol.challenge_id)) AS score,
        array_agg(sol.id) AS solve_ids,
        COALESCE(array_agg(sol.challenge_id), '{}'::uuid[]) AS chall_ids
    FROM solve_successes sol
    WHERE sol.user_id = users.id
) sol_info
ON true;

CREATE OR REPLACE VIEW user_view
AS SELECT
    u.*,
    ud.score,
    ud.solve_count,
    ud.last_solve,
    ud.last_solve_time,
    ud.chall_ids,

    (SELECT COUNT(*) FROM solve_attempts WHERE user_id = u.id) AS attempt_count
FROM users u
INNER JOIN user_data ud
ON u.id = ud.id;

CREATE UNIQUE INDEX user_data_id_idx on user_data (id);
