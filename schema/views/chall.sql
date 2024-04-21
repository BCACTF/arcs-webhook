DROP MATERIALIZED VIEW IF EXISTS challenge_data CASCADE;
CREATE MATERIALIZED VIEW IF NOT EXISTS challenge_data
AS SELECT
    chall.id,
    COALESCE(array_length(solve_ids, 1), 0) AS solve_count,

    get_solve_by_time(blood_time) AS blood_id,
    blood_time,

    get_solve_by_time(last_solve_time) AS last_solve,
    last_solve_time,

    COALESCE(user_ids, '{}'::uuid[]) AS user_ids,
    COALESCE(team_ids, '{}'::uuid[]) AS team_ids
FROM challenges chall
INNER JOIN LATERAL (
    SELECT
        MIN(solved_at) AS blood_time,
        MAX(solved_at) AS last_solve_time,

        array_agg(sol.id) AS solve_ids,
        array_agg(sol.user_id) AS user_ids,
        array_agg(sol.team_id) AS team_ids
    FROM solve_successes sol
    WHERE sol.challenge_id = chall.id
) sol_info
ON true;

CREATE OR REPLACE VIEW challenge_view
AS SELECT
    c.*,
    cd.solve_count AS __solve_count,
    cd.blood_id,
    cd.blood_time,
    cd.last_solve,
    cd.last_solve_time,
    cd.user_ids,
    cd.team_ids,
    (SELECT COUNT(*) FROM solve_attempts WHERE challenge_id = c.id) AS attempt_count
FROM challenges c
INNER JOIN challenge_data cd
ON c.id = cd.id;

CREATE UNIQUE INDEX challenge_data_id_idx on challenge_data (id);
