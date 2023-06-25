mod chall;
mod team;
mod user;

use serde::{Deserialize, Serialize};

pub use team::TeamQuery;
pub use user::{ UserQuery, Auth };
pub use chall::{ ChallQuery, Link, LinkType };




#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all = "snake_case")]
pub enum ToSql {
    User(UserQuery),
    Team(TeamQuery),
    Chall(ChallQuery),
    // Solve(SolveQuery),
}
