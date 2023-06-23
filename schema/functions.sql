CREATE OR REPLACE FUNCTION get_solves_chall(challenge_id uuid) RETURNS integer AS $$
    SELECT COUNT(attempt_id) AS result FROM solve_successes WHERE challenge_id = $1;
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION get_score_team(team_id uuid) RETURNS integer AS $$
    SELECT COALESCE(SUM(challenges.points), 0) AS result
    FROM challenges
        INNER JOIN solve_successes
        ON challenges.id = solve_successes.challenge_id
    WHERE solve_successes.team_id = $1;
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION get_score_user(user_id uuid) RETURNS integer AS $$
    SELECT COALESCE(SUM(challenges.points), 0) AS result
    FROM challenges
        INNER JOIN solve_successes
        ON challenges.id = solve_successes.challenge_id
    WHERE solve_successes.user_id = $1;
$$ LANGUAGE SQL;


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
$$ LANGUAGE plpgsql;


CREATE OR REPLACE FUNCTION update_all_user_scores() RETURNS void AS $$
    UPDATE users SET score = get_score_user(id);
$$ LANGUAGE SQL;
CREATE OR REPLACE FUNCTION update_all_team_scores() RETURNS void AS $$
    UPDATE teams SET score = get_score_team(id);
$$ LANGUAGE SQL;
CREATE OR REPLACE FUNCTION update_all_chall_solves() RETURNS void AS $$
    UPDATE teams SET score = get_score_team(id);
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION update_db_scores_solves() RETURNS void AS $$
    SELECT update_all_user_scores(), update_all_team_scores(), update_all_chall_solves();
$$ LANGUAGE SQL;



CREATE OR REPLACE FUNCTION insert_links_for_challenge(chall uuid, web text[], nc text[], admin text[], static text[]) RETURNS void AS $$
    INSERT INTO challenge_links (challenge_id, url, type)
        SELECT id, UNNEST(web), 'web'::public.link_type FROM challenges WHERE id = chall;
    
    INSERT INTO challenge_links (challenge_id, url, type)
        SELECT id, UNNEST(nc), 'nc'::public.link_type FROM challenges WHERE id = chall;

    INSERT INTO challenge_links (challenge_id, url, type)
        SELECT id, UNNEST(admin), 'admin'::public.link_type FROM challenges WHERE id = chall;

    INSERT INTO challenge_links (challenge_id, url, type)
        SELECT id, UNNEST(static), 'static'::public.link_type FROM challenges WHERE id = chall;
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION remove_links_for_challenge(chall uuid) RETURNS void AS $$
    DELETE FROM challenge_links WHERE challenge_id = $1;
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION replace_challenge_links(chall uuid, web text[], nc text[], admin text[], static text[]) RETURNS void AS $$
    SELECT remove_links_for_challenge($1);
    SELECT insert_links_for_challenge($1, $2, $3, $4, $5);
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION links_of_type(chall_id uuid, type link_type) RETURNS SETOF text AS $$
    SELECT url FROM challenge_links WHERE challenge_id = $1 AND type = $2;
$$ LANGUAGE SQL;

CREATE TYPE try_signin_ret AS ENUM (
    'not_found',
    'bad_auth',
    'authenticated'
);

CREATE OR REPLACE FUNCTION get_signin(user_id uuid) RETURNS text AS $$
    SELECT hashed_password FROM auth_name_pass WHERE user_id = user_id;
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION try_signin_oauth(user_id uuid, sub text, provider varchar(255)) RETURNS try_signin_ret AS $$
    SELECT CASE
        WHEN (SELECT COUNT(*) FROM auth_oauth WHERE user_id = $1) = 0 THEN 'not_found'::public.try_signin_ret
        WHEN (SELECT COUNT(*) FROM auth_oauth WHERE user_id = $1 AND sub = $2 AND provider_name = $3) != 1
            THEN 'bad_auth'::public.try_signin_ret
        ELSE 'authenticated'::public.try_signin_ret
    END result;
$$ LANGUAGE SQL;
