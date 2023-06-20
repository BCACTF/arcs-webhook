pub mod payloads;
pub mod handlers;

#[allow(unused_macros)]
pub mod logging {
    use arcs_logging_rs::with_target;
    with_target! { "Webhook" }
}


pub mod env {
    use arcs_env_rs::*;

    env_var_req!(FRONTEND_AUTH_TOKEN -> FRONTEND_TOKEN);
    env_var_req!(WEBHOOK_AUTH_TOKEN -> WEBHOOK_TOKEN);
    env_var_req!(DEPLOY_AUTH_TOKEN -> DEPLOY_TOKEN);
    
    env_var_req!(PORT);

    env_var_req!(FRONTEND_ADDRESS);
    env_var_req!(WEBHOOK_ADDRESS);
    env_var_req!(DEPLOY_ADDRESS);
        
    assert_req_env!(check_env_vars:
        PORT,
        FRONTEND_TOKEN, WEBHOOK_TOKEN, DEPLOY_TOKEN,
        FRONTEND_ADDRESS, WEBHOOK_ADDRESS, DEPLOY_ADDRESS
    );

    pub mod discord {
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

}

mod http_client {
    use lazy_static::lazy_static;
    use reqwest::Client;

    lazy_static! {
        pub static ref DEFAULT: Client = {
            Client::builder()
                .user_agent("ARCS webhook requests")
                .build()
                .unwrap()
        };
    }
}

