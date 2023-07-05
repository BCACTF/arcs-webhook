use webhook_rs::payloads::incoming::Incoming;

fn main() -> std::io::Result<()> {
    use schemars::schema_for;
    use serde_json::to_string_pretty;
    use std::{
        fs::{ write, create_dir_all },
        path::Path,
    };

    let incoming_schema = schema_for!(Incoming);

    create_dir_all(Path::new("./meta/"))?;

    write(
        Path::new("./meta/incoming.schema.json"),
        to_string_pretty(&incoming_schema).unwrap(),
    )
}
