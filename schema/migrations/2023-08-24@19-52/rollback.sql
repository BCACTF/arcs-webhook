ALTER TABLE teams DROP COLUMN last_tiebreaker_solve;

DROP FUNCTION update_scores_with_last_solve(uuid);

CREATE OR REPLACE FUNCTION update_scores_with_last_solve(success_id uuid) RETURNS void AS $$
    UPDATE users
    SET
        score = get_score_user(users.id),
        last_solve = CURRENT_TIMESTAMP,
        updated_at = CURRENT_TIMESTAMP
    WHERE users.id = (SELECT user_id FROM solve_successes WHERE id = $1);

    UPDATE teams
    SET
        score = get_score_team(teams.id),
        last_solve = CURRENT_TIMESTAMP,
        updated_at = CURRENT_TIMESTAMP
    WHERE teams.id = (SELECT team_id FROM solve_successes WHERE id = $1);

    UPDATE challenges
    SET
        solve_count = get_solves_chall(challenges.id),
        updated_at = CURRENT_TIMESTAMP
    WHERE challenges.id = (SELECT challenge_id FROM solve_successes WHERE id = $1);
$$ LANGUAGE SQL;
