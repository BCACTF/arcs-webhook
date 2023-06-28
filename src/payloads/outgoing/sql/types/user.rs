use serde::Serialize;
use uuid::Uuid;
use crate::sql::CiText;


#[derive(Debug, Clone, Serialize)]
pub struct SerializableUser {
    pub id: Uuid,
    pub email: CiText,
    pub name: CiText,

    pub team_id: Option<Uuid>,
    pub score: i32,
    pub last_solve: Option<u64>,
    
    pub eligible: bool,
    pub admin: bool,
}
impl From<User> for SerializableUser {
    fn from(User {
        id, email, name,
        team_id, score, last_solve,
        eligible, admin,
    }: User) -> Self {
        SerializableUser {
            id, email, name,
            team_id, score,
            eligible, admin,
            last_solve: last_solve.map(|dt| dt.timestamp() as u64),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(into = "SerializableUser")]
pub struct User {
    pub id: Uuid,
    pub email: CiText,
    pub name: CiText,

    pub team_id: Option<Uuid>,

    pub score: i32,
    pub last_solve: Option<chrono::NaiveDateTime>,
    
    pub eligible: bool,
    pub admin: bool,
}
