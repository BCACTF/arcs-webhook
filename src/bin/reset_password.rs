use std::io::Write;

use uuid::Uuid;
use webhook_rs::passwords::{salt, ARGON2_CONFIG};

fn main() {

    print!("Enter team ID: ");
    std::io::stdout().flush().expect("Failed to flush stdout.");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read team ID from stdin.");

    let team_id = input.trim().parse::<Uuid>().expect("Failed to parse team ID as UUID.");
    input.clear();


    print!("Enter team password: ");
    std::io::stdout().flush().expect("Failed to flush stdout.");

    std::io::stdin().read_line(&mut input).expect("Failed to read new password from stdin.");
    let password = input.trim().to_string();


    let salt = salt().expect("Failed to generate salt for team password.");

    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &ARGON2_CONFIG).expect("Failed to hash team password.");

    println!(
        "UPDATE teams SET hashed_password = '{}' WHERE id = '{}';",
        format!("{hash:?}").trim_matches('"'),
        team_id.hyphenated()
    );
}
