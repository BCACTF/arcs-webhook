use std::io::Write;

use sqlx::Row;
use uuid::Uuid;
use webhook_rs::passwords::ARGON2_CONFIG;

enum TeamIdent {
    ID(Uuid),
    Name(String),
}

#[tokio::main]
async fn main() {

    print!("Enter team ID or Name: ");
    std::io::stdout().flush().expect("Failed to flush stdout.");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read team name/ID from stdin.");

    let team_ident = if let Ok(id) = input.trim().parse::<Uuid>() {
        TeamIdent::ID(id)
    } else {
        TeamIdent::Name(input.trim().to_string())
    };
    input.clear();


    print!("Enter team password: ");
    std::io::stdout().flush().expect("Failed to flush stdout.");

    std::io::stdin().read_line(&mut input).expect("Failed to read password from stdin.");
    let password = input.trim().to_string();

    let pg_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL not set in .env file.");
    let pool = sqlx::PgPool::connect(&pg_url).await.expect("Failed to connect to database.");

    let mut connection = pool.acquire().await.expect("Failed to acquire connection to database.");

    let team_id = match team_ident {
        TeamIdent::ID(id) => id,
        TeamIdent::Name(name) => {
            let team = sqlx::query("SELECT id FROM teams WHERE name = $1;")
                .bind(&name)
                .fetch_one(&mut connection)
                .await
                .expect("Failed to fetch team ID from database.");

            team.get("id")
        }
    };

    let hash: String = sqlx::query_scalar!("SELECT hashed_password FROM teams WHERE id = $1;", team_id)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch hashed password from database.");

    let result = argon2::verify_encoded(&hash, password.as_bytes()).expect("Failed to verify password. (ARGON2 error)");

    if result {
        print!("\x1b[32m");
        print!("Password is valid.");
    } else {
        print!("\x1b[1;5;31m");
        print!("PASSWORD IS INVALID!!!");
    }
    println!("\x1b[0m");
}
