ALTER TABLE teams ADD COLUMN last_tiebreaker_solve timestamp(0) without time zone;

DROP FUNCTION update_scores_with_last_solve(uuid);
CREATE FUNCTION update_scores_with_last_solve(success_id uuid) RETURNS void AS $$
    DECLARE
        solve_time timestamp(0) without time zone := (SELECT solved_at FROM solve_successes WHERE id = success_id);

        user_id uuid := (SELECT user_id FROM solve_successes WHERE id = success_id);
        team_id uuid := (SELECT team_id FROM solve_successes WHERE id = success_id);
        chal_id uuid := (SELECT challenge_id FROM solve_successes WHERE id = success_id);

        tiebreaking boolean := (SELECT tiebreaker FROM challenges WHERE id = chal_id);
    BEGIN
    UPDATE users
        SET
            score = get_score_user(users.id),
            last_solve = solve_time,
            updated_at = CURRENT_TIMESTAMP
        WHERE users.id = user_id;

        UPDATE teams
        SET
            score = get_score_team(teams.id),
            last_solve = solve_time,
            updated_at = CURRENT_TIMESTAMP
        WHERE teams.id = team_id;

        UPDATE challenges
        SET
            solve_count = get_solves_chall(challenges.id),
            updated_at = CURRENT_TIMESTAMP
        WHERE challenges.id = chal_id;

        IF tiebreaking THEN
            UPDATE teams
            SET last_tiebreaker_solve = solve_time
            WHERE teams.id = team_id;
        END IF;
    END;
$$ LANGUAGE plpgsql;

SELECT update_scores_with_last_solve(id) FROM solve_successes;
