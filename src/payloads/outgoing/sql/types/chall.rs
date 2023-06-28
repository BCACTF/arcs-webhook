use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::sql::CiText;

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