use std::fs::File;
use std::path::Path;
use std::process::exit;

use xwingtmg2_inventory_rs::{
    expansions::{self, ItemType},
    xwingdata2,
    yasb2::{self},
    Collection, PilotRecord, ShipRecord, UpgradeRecord,
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

    let yasb_coll = match yasb2::load_collection_file(Path::new("collection.json")) {
        Ok(c) => c,
        Err(e) => {
            println!("{:?}", e);
            exit(1)
        }
    };

    let (skus, missing) = yasb_coll.expansion_skus(&expansions);

    println!("Not found expansions (probably 1.0, but for debugging):");
    for n in missing {
        println!("- {}", n);
    }

    let collection = Collection {
        skus,
        singles: yasb_coll.singles_as_xws(),
    };

    let (inventory, missing) = collection.inventory(expansions);
    if !missing.is_empty() {
        println!("YASB module added a not found expansion without reporting:");
        for n in missing {
            println!("- {}", n);
        }
    }

    // TODO: Can some this to_owned() just be references?
    // TODO: Find a CSV serializer, but for now, dump as json and use jq

    ships_json(&inventory, &xwd_data);
    pilots_json(&inventory, &xwd_data);
    upgrades_json(&inventory, &xwd_data);
}

fn pilots_json(
    inventory: &std::collections::BTreeMap<expansions::Item, u32>,
    xwd_data: &xwingdata2::Data,
) {
    let mut records = vec![];
    for (item, count) in inventory {
        if item.r#type != ItemType::Pilot {
            continue;
        }
        match xwd_data.get_pilot(&item.xws) {
            Some((s, p)) => records.push(crate::PilotRecord {
                faction: s.faction.to_owned(),
                ship: s.name.to_owned(),
                name: p.name.to_owned(),
                xws: p.xws.to_owned(),
                initiative: p.initiative,
                count: *count,
            }),
            None => println!("Pilot not found: {}", item.xws),
        };
    }
    println!(
        "Total {} pilots, {} unique",
        records.iter().fold(0, |acc, r| acc + r.count),
        records.len()
    );

    let f = File::create("pilots.json").unwrap();
    match serde_json::to_writer(f, &records) {
        Ok(_) => println!("pilots.json written"),
        Err(err) => println!("pilots.json error: {}", err),
    };
}

fn upgrades_json(
    inventory: &std::collections::BTreeMap<expansions::Item, u32>,
    xwd_data: &xwingdata2::Data,
) {
    let mut records = vec![];
    for (item, count) in inventory {
        if item.r#type != ItemType::Upgrade {
            continue;
        }
        match xwd_data.get_upgrade(&item.xws) {
            Some(u) => records.push(UpgradeRecord::new(u, *count)),
            None => println!("Upgrade not found: {}", &item.xws),
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
}

fn ships_json(
    inventory: &std::collections::BTreeMap<expansions::Item, u32>,
    xwd_data: &xwingdata2::Data,
) {
    let mut records = vec![];
    for (item, count) in inventory {
        if item.r#type != ItemType::Ship {
            continue;
        }
        match xwd_data.get_ship(&item.xws) {
            Some(u) => records.push(ShipRecord::new(u, *count)),
            None => println!("ship not found: {}", &item.xws),
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
