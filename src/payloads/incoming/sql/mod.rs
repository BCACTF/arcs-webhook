mod chall;
mod solve;
mod team;
mod user;
mod attempts;

use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};

pub use chall::{ ChallQuery, Link, LinkType };
pub use solve::SolveQuery;
pub use team::TeamQuery;
pub use user::{ UserQuery, Auth };
pub use attempts::AttemptQuery;




#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "__type", rename_all = "snake_case", content = "details")]
pub enum ToSql {
    User(UserQuery),
    Team(TeamQuery),
    Chall(ChallQuery),
    Solve(SolveQuery),
    Attempt(AttemptQuery),
}
