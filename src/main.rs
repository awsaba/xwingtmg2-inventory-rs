use std::fs::File;
use std::path::Path;
use std::process::exit;

use xwingtmg2_inventory_rs::{
    expansions::{self, ItemType},
    xwingdata2,
    yasb2::{self},
    PilotRecord, ShipRecord, UpgradeRecord,
};

fn main() {
    let xwd_data = match xwingdata2::load_from_manifest(Path::new("xwing-data2")) {
        Ok(d) => d,
        Err(e) => {
            println!("{:?}", e);
            exit(1)
        }
    };
    //println!("{:?}", xws_data);

    let expansions = match expansions::load_expansions() {
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

    let xws_collection = collection.to_xws_collection(&expansions);

    println!("Not found expansions (probably 1.0, but for debugging):");
    for n in xws_collection.missing_expansions {
        println!("- {}", n);
    }

    println!("Not found singles (for YASB, this is usually old renames in the collections data):");
    for i in xws_collection.missing_singles {
        println!("- {}", i.xws);
    }

    // TODO: Can some this to_owned() just be references?
    let mut records = vec![];
    for ic in &xws_collection.item_counts {
        if ic.item.r#type != ItemType::Pilot {
            continue;
        }
        match xwd_data.get_pilot(&ic.item.xws) {
            Some((s, p)) => records.push(crate::PilotRecord {
                faction: s.faction.to_owned(),
                ship: s.name.to_owned(),
                name: p.name.to_owned(),
                xws: p.xws.to_owned(),
                initiative: p.initiative,
                count: ic.count,
            }),
            None => println!("Pilot not found: {}", ic.item.xws),
        };
    }
    println!(
        "Total {} pilots, {} unique",
        records.iter().fold(0, |acc, r| acc + r.count),
        records.len()
    );

    // TODO: Find a CSV serializer, but for now, dump as json and use jq
    let f = File::create("pilots.json").unwrap();
    match serde_json::to_writer(f, &records) {
        Ok(_) => println!("pilots.json written"),
        Err(err) => println!("pilots.json error: {}", err),
    };

    let mut records = vec![];
    for ic in &xws_collection.item_counts {
        if ic.item.r#type != ItemType::Upgrade {
            continue;
        }
        match xwd_data.get_upgrade(&ic.item.xws) {
            Some(u) => records.push(UpgradeRecord::new(u, ic.count)),
            None => println!("Upgrade not found: {}", &ic.item.xws),
        };
    }
    println!(
        "Total {} upgrades, {} unique",
        records.iter().fold(0, |acc, r| acc + r.count),
        records.len()
    );

    let f = File::create("upgrades.json").unwrap();
    match serde_json::to_writer(f, &records) {
        Ok(_) => println!("upgrades.json written"),
        Err(err) => println!("upgrades.json error: {}", err),
    };

    let mut records = vec![];
    for ic in &xws_collection.item_counts {
        if ic.item.r#type != ItemType::Ship {
            continue;
        }
        match xwd_data.get_ship(&ic.item.xws) {
            Some(u) => records.push(ShipRecord::new(u, ic.count)),
            None => println!("ship not found: {}", &ic.item.xws),
        };
    }
    println!(
        "Total {} ships, {} unique",
        records.iter().fold(0, |acc, r| acc + r.count),
        records.len()
    );

    let f = File::create("ships.json").unwrap();
    match serde_json::to_writer(f, &records) {
        Ok(_) => println!("ships.json written"),
        Err(err) => println!("ships.json error: {}", err),
    };
}
