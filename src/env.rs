//! General purpose environment variables for the webhook server.
//! 
//! Check out [discord] and [sql] for more specific environment variables,
//! and check out [checks] for how to check the variables at runtime.
//! 
//! Auth variables are in an extenally-inaccessible module [crate::auth].

use arcs_env_rs::*;

env_var_req!(PORT);

env_var_req!(FRONTEND_ADDRESS);
env_var_req!(WEBHOOK_ADDRESS);
env_var_req!(DEPLOY_ADDRESS);
    
assert_req_env!(check_env_vars:
    PORT,
    FRONTEND_ADDRESS, WEBHOOK_ADDRESS, DEPLOY_ADDRESS
);

pub (crate) mod discord {
    //! URLs and roles for disseminating webhook messages to different
    //! groups of people.
    //! 
    //! These are:
    //! - Admins ([admin_url], [admin_role])
    //! - Challenge writers/authors ([chall_writer_url], [chall_writer_role])
    //! - CTF participants ([participant_url], [participant_role])

    use arcs_env_rs::*;

    env_var_req!(DISCORD_ADMIN_WEBHOOK_URL -> ADMIN_URL);
    env_var_req!(DISCORD_CHALL_WRITER_WEBHOOK_URL -> CHALL_WRITER_URL);
    env_var_req!(DISCORD_PARTICIPANT_URL -> PARTICIPANT_URL);

    env_var_req!(DISCORD_ADMIN_ROLE_ID -> ADMIN_ROLE);
    env_var_req!(DISCORD_CHALL_WRITER_ROLE_ID -> CHALL_WRITER_ROLE);
    env_var_req!(DISCORD_PARTICIPANT_ROLE_ID -> PARTICIPANT_ROLE);

    assert_req_env!(
        check_env_vars:
            ADMIN_URL,  CHALL_WRITER_URL,  PARTICIPANT_URL,
            ADMIN_ROLE, CHALL_WRITER_ROLE, PARTICIPANT_ROLE
    );
}

pub (crate) mod sql {
    //! [Database name][db_name] + [username] for the postgres instance.
    //! 
    //! A connection to the db can be acquired through the [crate::sql]
    //! module.


    use arcs_env_rs::*;

    env_var_req!(SQL_DB_NAME -> DB_NAME);
    env_var_req!(SQL_USERNAME -> USERNAME);

    assert_req_env!(
        check_env_vars:
            DB_NAME,
            USERNAME
    );
}

pub mod checks {
    //! Functions to assert the presence and validity of the environment
    //! variables at runtime.
    //! 
    //! The individual functions are:
    //! - [main]
    //! - [discord]
    //! - [sql]
    //! - [auth]
    
    pub use super::check_env_vars as main;
    pub use super::discord::check_env_vars as discord;
    pub use super::sql::check_env_vars as sql;
    pub use crate::auth::check_env_vars as auth;
}