use std::borrow::Cow;

use serde::Serialize;
use uuid::Uuid;

use crate::handlers::OutgoingErr;

// TODO: Fix this
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "__type", rename_all = "snake_case", content = "data")]
pub enum FromSql {
    Chall(Chall),
    ChallArr(Vec<Chall>),
    
    Team(Team),
    TeamArr(Vec<Team>),
    
    User(User),
    UserArr(Vec<User>),
    
    // Solves(),

    Availability(bool),
    AuthStatus(bool),
}

#[derive(Debug, Clone, Serialize)]
pub enum FromSqlErr {
    OtherServerError(Cow<'static, str>),
    DatabaseError,
    Auth,
    DoesNotExist(Uuid),
    NameDoesNotExist(String),
}

impl From<sqlx::Error> for FromSqlErr {
    fn from(_: sqlx::Error) -> Self {
        Self::DatabaseError
    }
}

impl OutgoingErr for FromSqlErr {
    fn body(self) -> Result<serde_json::Value, String> {
        match self {
            Self::OtherServerError(description) => Ok(serde_json::json!({
                "err": "An unknown server error occured.",
                "info": description,
            })),
            Self::DatabaseError => Ok(serde_json::json!({
                "err": "Unexpected database error encountered.",
            })),
            Self::Auth => Ok(serde_json::json!({
                "err": "Unauthorized access.",
            })),
            Self::DoesNotExist(id) => Ok(serde_json::json!({
                "err": "This id does not exist.",
                "id": id,
            })),
            Self::NameDoesNotExist(name) => Ok(serde_json::json!({
                "err": "This name does not exist.",
                "name": name,
            })),
        }
    }
    fn status_code(&self) -> u16 {
        match self {
            Self::OtherServerError(_) | Self::DatabaseError => 500,
            Self::DoesNotExist(_) | Self::NameDoesNotExist(_) => 404,
            Self::Auth => 403,
        }
    }
}

mod types {
    use serde::{Serialize, Deserialize};
    use uuid::Uuid;

    use crate::sql::CiText;

    #[derive(Debug, Clone, Serialize)]
    pub struct SerializableTeam {
        pub id: Uuid,
        pub name: CiText,
        pub last_solve: Option<u64>,
        pub eligible: bool,
        pub affiliation: Option<String>,
    }
    impl From<Team> for SerializableTeam {
        fn from(Team { id, name, last_solve, eligible, affiliation }: Team) -> Self {
            SerializableTeam {
                id, name, eligible, affiliation,
                last_solve: last_solve.map(|dt| dt.timestamp() as u64),
            }
        }
    }

    #[derive(Debug, Clone, Serialize)]
    #[serde(into = "SerializableTeam")]
    pub struct Team {
        pub id: Uuid,
        pub name: CiText,
        pub last_solve: Option<chrono::NaiveDateTime>,
        pub eligible: bool,
        pub affiliation: Option<String>,
    }

    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Links {
        nc: Vec<String>,
        web: Vec<String>,
        admin: Vec<String>,
        #[serde(rename = "static")]
        static_links: Vec<String>, 
    }
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SerializableChall {
        pub id: Uuid,
        pub name: CiText,
        pub description: String,
        pub points: i32,
        pub authors: Vec<String>,
        pub hints: Vec<String>,
        pub categories: Vec<String>,
        pub tags: Vec<String>,
        pub solve_count: i32,
        pub visible: bool,
        pub source_folder: String,


        pub links: Links,
    }
    impl From<SerializableChall> for Chall {
        fn from(SerializableChall {
            id, name, description, points,
            authors, hints, categories, tags,
            solve_count, visible, source_folder,
            links: Links {
                nc: links_nc,
                web: links_web,
                admin: links_admin,
                static_links: links_static,
            },
        }: SerializableChall) -> Self {
            Chall {
                id, name, description, points,
                authors, hints, categories, tags,
                solve_count, visible, source_folder,
                links_nc, links_web, links_admin, links_static,
            }
        }
    }
    impl From<Chall> for SerializableChall {
        fn from(Chall {
            id, name, description, points,
            authors, hints, categories, tags,
            solve_count, visible, source_folder,
            links_nc: nc, links_web: web, links_admin: admin, links_static: static_links,
        }: Chall) -> Self {
            SerializableChall {
                id, name, description, points,
                authors, hints, categories, tags,
                solve_count, visible, source_folder,
                links: Links { nc, web, admin, static_links },
            }
        }
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(into = "SerializableChall", from = "SerializableChall")]
    pub struct Chall {
        pub id: Uuid,
        pub name: CiText,
        pub description: String,
        pub points: i32,
        pub authors: Vec<String>,
        pub hints: Vec<String>,
        pub categories: Vec<String>,
        pub tags: Vec<String>,
        pub solve_count: i32,
        pub visible: bool,
        pub source_folder: String,


        pub links_nc: Vec<String>,
        pub links_web: Vec<String>,
        pub links_admin: Vec<String>,
        pub links_static: Vec<String>,
    }



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
}
pub use types::{ Team, Chall, User };

