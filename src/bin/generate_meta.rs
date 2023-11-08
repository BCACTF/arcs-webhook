use webhook_rs::payloads::{
    incoming::Incoming,
    outgoing::Outgoing,
};

fn main() -> std::io::Result<()> {
    use schemars::schema_for;
    use serde_json::to_string_pretty;
    use std::{
        fs::{ write, create_dir_all },
        path::Path,
    };

    let incoming_schema = schema_for!(Incoming);
    let outgoing_schema = schema_for!(Outgoing);

    create_dir_all(Path::new("./meta/"))?;

    write(
        Path::new("./meta/incoming.schema.json"),
        to_string_pretty(&incoming_schema).unwrap(),
    )?;

    write(
        Path::new("./meta/outgoing.schema.json"),
        to_string_pretty(&outgoing_schema).unwrap(),
    )?;

    Ok(())
}
