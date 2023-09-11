use std::path::Path;
use std::process::exit;
use std::{fs::File, path::PathBuf};

use strum::EnumString;
use xwingtmg2_inventory_rs::Records;
use xwingtmg2_inventory_rs::{expansions::Catalog, xwingdata2::Data, yasb2, Collection};

const HELP: &str = "\
xwingtmg2-inventory

USAGE:
  xwingtmg2-inventory [options]

FLAGS:
  -h, --help            Prints help information
  -f, --format          JSON or XLSX (default: JSON)
  -c, --collection      A YASB collection in YASB's json format
  -o, --only-owned      Includes all known expansions and contents
";

#[derive(PartialEq, EnumString)]
enum Format {
    #[strum(serialize = "json", serialize = "JSON")]
    Json,
    #[strum(serialize = "xlsx", serialize = "XLSX")]
    Xlsx,
}

struct Args {
    only_owned: bool,
    collection_json: Option<PathBuf>,
    format: Format,
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args = Args {
        only_owned: pargs.contains(["-l", "--only-owned"]),
        collection_json: pargs.opt_value_from_os_str(["-c", "--collection"], parse_path)?,
        format: pargs
            .opt_value_from_str::<_, Format>(["-f", "--format"])?
            .unwrap_or(Format::Json),
    };

    // It's up to the caller what to do with the remaining arguments.
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {:?}.", remaining);
    }

    Ok(args)
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
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

    let yasb_coll = match args.collection_json {
        None => yasb2::Collection::default(),
        Some(p) => match yasb2::Collection::load(&p) {
            Ok(c) => c,
            Err(e) => {
                println!("{:?}", e);
                exit(1)
            }
        },
    };

    let (mut skus, missing) = yasb_coll.expansion_skus(&catalog);

    println!("Not found expansions (probably 1.0, but for debugging):");
    for n in missing {
        println!("- {}", n);
    }

    if !args.only_owned {
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
    // FIXME: This is doing a bunch of stuff twice for xlsx generatino, but
    // the stats are nice, so keeping it for now.
    let records = Records::build(&inventory, &data, &catalog);
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

    match args.format {
        Format::Json => {
            let f = File::create("inventory.json").unwrap();
            match serde_json::to_writer(f, &records) {
                Ok(_) => println!("inventory.json written"),
                Err(err) => println!("inventory.json error: {}", err),
            }
        }
        Format::Xlsx => {
            match xwingtmg2_inventory_rs::generate_xls(&catalog, &data, &collection, &inventory) {
                Ok(_) => println!("xlsx written"),
                Err(err) => println!("xlsx error: {}", err),
            }
        }
    };
}
