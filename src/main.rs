use serde::Serialize;
use std::fs::File;
use std::path::Path;
use std::process::exit;

mod xwingdata2;
mod yasb2;

/// PilotRecord has fields that I want to sort by so that I can organize my
/// collection, either in binders or boxes.
#[derive(Serialize, Debug)]
struct PilotRecord {
    pub faction: String,
    pub ship: String,
    pub name: String,
    pub initiative: u32,

    pub count: u32,
}

/// UpgradeRecord are the fields I sort my collection by.
#[derive(Serialize, Debug)]
struct UpgradeRecord {
    pub r#type: String,
    pub faction_restriction: String,
    pub name: String,
    pub size_restriction: String,
    pub ship_restriction: String,

    pub count: u32,
}

fn main() {
    println!("Hello, world!");

    let xwd_data = match xwingdata2::load_from_manifest(Path::new("xwing-data2")) {
        Ok(d) => d,
        Err(e) => {
            println!("{:?}", e);
            exit(1)
        }
    };
    //println!("{:?}", xws_data);

    let expansions = match yasb2::load_expansions() {
        Ok(e) => e,
        Err(e) => {
            println!("{:?}", e);
            exit(2)
        }
    };

    let collection = match yasb2::load_collection_file(Path::new("collection.json")) {
        Ok(c) => c,
        Err(e) => {
            println!("{:?}", e);
            exit(1)
        }
    };

    let (pilots, upgrades, not_found) = yasb2::collection_to_xws_count(&collection, &expansions);

    println!("Total unique pilots: {}", pilots.len());
    println!("Total unique upgrades: {}", upgrades.len());

    println!("Not found factions (probably 1.0, but for debugging):");
    for n in not_found {
        println!("{}", n);
    }

    // TODO: Can some this to_owned() just be references?
    let mut records = vec![];
    for (n, c) in pilots {
        match xwd_data.get_pilot(&n) {
            Some((s, p)) => records.push(PilotRecord {
                faction: s.faction.to_owned(),
                ship: s.name.to_owned(),
                name: p.name.to_owned(),
                initiative: p.initiative,
                count: c,
            }),
            None => println!("Pilot not found: {}", &n),
        };
    }

    // TODO: Find a CSV serializer, but for now, dump as json and use jq
    let f = File::create("pilots.json").unwrap();
    match serde_json::to_writer(f, &records) {
        Ok(_) => println!("pilots.json written"),
        Err(err) => println!("pilots.json error: {}", err),
    };

    let mut records = vec![];
    for (n, c) in upgrades {
        match xwd_data.get_upgrade(&n) {
            Some(u) => records.push(UpgradeRecord {
                name: u.name.to_owned(),
                count: c,
                r#type: u
                    .sides
                    .first()
                    .map(|s| s.r#type.to_owned())
                    .unwrap_or("not found".to_owned())
                    .to_owned(),
                faction_restriction: "".to_string(),
                size_restriction: "".to_string(),
                ship_restriction: "".to_string(),
            }),
            None => println!("Upgrade not found: {}", &n),
        };
    }

    let f = File::create("upgrades.json").unwrap();
    match serde_json::to_writer(f, &records) {
        Ok(_) => println!("upgrades.json written"),
        Err(err) => println!("upgrades.json error: {}", err),
    };
}
