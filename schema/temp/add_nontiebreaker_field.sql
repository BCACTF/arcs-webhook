ALTER TABLE challenges
ADD COLUMN non_tie_breaker boolean
DEFAULT false NOT NULL;

CREATE OR REPLACE FUNCTION update_scores_with_last_solve(success_id uuid) RETURNS void AS $$
    DECLARE
        was_tie_breaker boolean := (SELECT (c.non_tie_breaker = false) AS was_tie_breaker FROM solve_successes as s JOIN challenges as c ON s.challenge_id = c.id WHERE s.id = $1);
    BEGIN
        UPDATE users
        SET
            score = get_score_user(users.id),
            updated_at = CURRENT_TIMESTAMP
        WHERE users.id = (SELECT user_id FROM solve_successes WHERE id = $1);

        UPDATE teams
        SET
            score = get_score_team(teams.id),
            updated_at = CURRENT_TIMESTAMP
        WHERE teams.id = (SELECT team_id FROM solve_successes WHERE id = $1);

        UPDATE challenges
        SET
            solve_count = get_solves_chall(challenges.id),
            updated_at = CURRENT_TIMESTAMP
        WHERE challenges.id = (SELECT challenge_id FROM solve_successes WHERE id = $1);

        IF was_tie_breaker THEN 
            UPDATE users
            SET last_solve = CURRENT_TIMESTAMP
            WHERE users.id = (SELECT user_id FROM solve_successes WHERE id = $1);

            UPDATE teams
            SET last_solve = CURRENT_TIMESTAMP
            WHERE teams.id = (SELECT team_id FROM solve_successes WHERE id = $1);
        END IF;
    END;
$$ LANGUAGE plpgsql VOLATILE;

CREATE OR REPLACE FUNCTION do_solve_attempt(att_user_id uuid, att_team_id uuid, att_challenge_id uuid, att_guess varchar(255)) RETURNS RECORD AS $$
    DECLARE
        guess_correct boolean := (SELECT flag = att_guess AS correct FROM challenges WHERE id = att_challenge_id);
        already_solved boolean := (SELECT COUNT(id) FROM solve_successes WHERE challenge_id=att_challenge_id AND team_id=att_team_id);
        attempt_id uuid := uuid_generate_v4();
        success_entry_id uuid := uuid_generate_v4();

        output RECORD;
    BEGIN
        INSERT INTO solve_attempts (id, flag_guess, correct, user_id, team_id, challenge_id)
        VALUES (attempt_id, att_guess, guess_correct, att_user_id, att_team_id, att_challenge_id);
        
        IF guess_correct AND NOT already_solved THEN 
            INSERT INTO solve_successes (id, attempt_id, user_id, team_id, challenge_id)
            VALUES (success_entry_id, attempt_id, att_user_id, att_team_id, att_challenge_id);

            PERFORM update_scores_with_last_solve(success_entry_id);
        END IF;
        
        output := (attempt_id, guess_correct, already_solved);
        RETURN output;
    END;
$$ LANGUAGE plpgsql VOLATILE;
