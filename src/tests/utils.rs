pub fn setup_env() {
    dotenvy::dotenv().unwrap();

    std::env::set_var("PORT", "3000");

    std::env::set_var("FRONTEND_AUTH_TOKEN", "F".repeat(64));
    std::env::set_var("WEBHOOK_AUTH_TOKEN", "W".repeat(64));
    std::env::set_var("DEPLOY_AUTH_TOKEN", "D".repeat(64));
    std::env::set_var("ALLOWED_OAUTH_TOKEN", "A".repeat(64));

    std::env::set_var("FRONTEND_ADDRESS", "");
    std::env::set_var("WEBHOOK_ADDRESS", "");
    std::env::set_var("DEPLOY_ADDRESS", "");

    std::env::set_var("DEPLOY_ADDRESS", "");
}

pub fn rand_chars(len: usize, alphabet: &str) -> String {
    let mut chars = vec!['\0'; len];
    rand::Rng::fill(&mut rand::thread_rng(), chars.as_mut_slice());
    chars
        .into_iter()
        .map(|c| alphabet.chars().nth(c as usize % alphabet.len()).unwrap_or('\0'))
        .collect()
}

