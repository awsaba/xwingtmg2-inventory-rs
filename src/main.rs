use std::fs::File;
use std::path::Path;
use std::process::exit;

use serde::Serialize;
use xwingtmg2_inventory_rs::{
    expansions::{Catalog, ItemType},
    xwingdata2::Data,
    yasb2, Collection, Inventory, PilotRecord, ShipRecord, UpgradeRecord,
};

const HELP: &str = "\
xwingtmg2-inventory

USAGE:
  xwingtmg2-inventory [options]

FLAGS:
  -h, --help            Prints help information
  -l, --list-missing    Includes all known expansions and there contents with a count of 0
";

struct Args {
    list_missing: bool,
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = Args {
        list_missing: pargs.contains(["-l", "--list-missing"]),
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

// TODO: Figure out what is generic here
#[derive(Default, Serialize)]
struct Records {
    pub ships: Vec<ShipRecord>,
    pub pilots: Vec<PilotRecord>,
    pub upgrades: Vec<UpgradeRecord>,
}

fn main() {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            std::process::exit(1);
        }
    };

    let data = match Data::load_from_manifest(Path::new("xwing-data2")) {
        Ok(d) => d,
        Err(e) => {
            println!("{:?}", e);
            exit(1)
        }
    };
    //println!("{:?}", xws_data);

    let catalog = match Catalog::load() {
        Ok(e) => e,
        Err(e) => {
            println!("{:?}", e);
            exit(2)
        }
    };

    let yasb_coll = match yasb2::Collection::load(Path::new("collection.json")) {
        Ok(c) => c,
        Err(e) => {
            println!("{:?}", e);
            exit(1)
        }
    };

    let (mut skus, missing) = yasb_coll.expansion_skus(&catalog);

    println!("Not found expansions (probably 1.0, but for debugging):");
    for n in missing {
        println!("- {}", n);
    }

    if args.list_missing {
        for sku in catalog.expansions.keys() {
            if skus.get(sku).is_none() {
                skus.insert(sku.to_owned(), 0);
            }
        }
    }

    let collection = Collection {
        skus,
        singles: yasb_coll.singles_as_xws(),
    };

    let (inventory, missing) = collection.inventory(&catalog);
    if !missing.is_empty() {
        println!("YASB module added a not found expansion without reporting:");
        for n in missing {
            println!("- {}", n);
        }
    }

    // TODO: Can some this to_owned() just be references?
    // TODO: Find a CSV serializer, but for now, dump as json and use jq
    let records = assemble_records(&inventory, &data, &catalog);
    println!(
        "Total {} ships, {}/{} unique",
        records.ships.iter().fold(0, |acc, r| acc + r.count),
        records
            .ships
            .iter()
            .fold(0, |acc, r| if r.count > 0 { acc + 1 } else { acc }),
        records.ships.len(),
    );
    println!(
        "Total {} pilots, {}/{} unique",
        records.pilots.iter().fold(0, |acc, r| acc + r.count),
        records
            .pilots
            .iter()
            .fold(0, |acc, r| if r.count > 0 { acc + 1 } else { acc }),
        records.pilots.len(),
    );
    println!(
        "Total {} upgrades, {}/{} unique",
        records.upgrades.iter().fold(0, |acc, r| acc + r.count),
        records
            .upgrades
            .iter()
            .fold(0, |acc, r| if r.count > 0 { acc + 1 } else { acc }),
        records.upgrades.len(),
    );

    let f = File::create("inventory.json").unwrap();
    match serde_json::to_writer(f, &records) {
        Ok(_) => println!("inventory.json written"),
        Err(err) => println!("inventory.json error: {}", err),
    };
}

fn assemble_records(inventory: &Inventory, data: &Data, catalog: &Catalog) -> Records {
    let mut records = Records {
        ..Default::default()
    };

    for (item, count) in inventory {
        match &item.r#type {
            ItemType::Ship => {
                match ShipRecord::build(&item.xws, *count, data, catalog) {
                    Ok(r) => records.ships.push(r),
                    Err(_) => println!("ship not found: {}", &item.xws),
                };
            }
            ItemType::Pilot => {
                match PilotRecord::build(&item.xws, *count, data, catalog) {
                    Ok(r) => records.pilots.push(r),
                    Err(_) => println!("pilot not found: {}", &item.xws),
                };
            }
            ItemType::Upgrade => {
                match UpgradeRecord::build(&item.xws, *count, data, catalog) {
                    Ok(u) => records.upgrades.push(u),
                    Err(_) => println!("Upgrade not found: {}", &item.xws),
                };
            }
            _ => (),
        };
    }
    records
}
