//! A Rust library for keeping track of an X-Wing: The Miniatures Game, 2nd
//! edition collection, based on product skus and xws id.
//!
//! It currently suppports importing a collection from `yasb.app` and producing
//! a usable JSON document.
//!
//! The "*Record" types are subsets of the xwing-data2 info that is relevant
//! to me for sorting my collection after importing into a spreadsheet.
//!
//! See the project README.md for example usage of the included CLI utility.
use crate::expansions::Item;
use crate::xwingdata2::Restriction;
pub mod expansions;
pub mod xwingdata2;
pub mod yasb2;

use expansions::{Catalog, Expansion, ItemCount, ItemType, SKU};
use serde::{Deserialize, Serialize};
use xwingdata2::Data;

use rust_xlsxwriter::utility::row_col_to_cell;
use rust_xlsxwriter::{Table, TableStyle, Workbook, XlsxError};

use std::collections::BTreeMap;

#[derive(Debug)]
pub enum ErrorKind {
    NotFound,
}

/// A collection is:
/// - A list of expansions and their counts, indexed by SKU
/// - A list of additional `singles` identified by their type and xws id.
///
/// Minimal error checking is done by the collection itself, it mostly defines
/// a tool agnostic, unambigous definition of a collection.
#[derive(Default, Serialize, Deserialize)]
pub struct Collection {
    pub skus: BTreeMap<SKU, u32>,
    pub singles: BTreeMap<Item, u32>,
}

/// An Inventory is a just a count of Items, where Items have just enough
/// information to look them up in xwing-data2 or an catalog of expansion
/// contents.
pub type Inventory = BTreeMap<Item, u32>;

impl Collection {
    /// Produce a count of all items in expansions and add them to the singles.
    ///
    /// Returns a list of expansions that weren't found in the catalog.
    pub fn inventory(&self, catalog: &expansions::Catalog) -> (Inventory, Vec<String>) {
        let mut inventory = self.singles.clone();
        let mut missing_expansions = vec![];

        for (sku, c) in &self.skus {
            match catalog.expansions.get(sku) {
                Some(expansion) => {
                    for item_count in &expansion.contents {
                        let total =
                            inventory.get(&item_count.item).unwrap_or(&0) + c * item_count.count;
                        inventory.insert(item_count.item.clone(), total);
                    }
                }
                None => missing_expansions.push(sku.to_owned()),
            };
        }
        (inventory, missing_expansions)
    }
}

/// This is the full ship as defined by the expansions.
///
/// TODO: Add a "miniature/chassis" type compatibility that reflects usability
/// per tournament regulations.
#[derive(Serialize, Debug)]
pub struct ShipRecord {
    pub name: String,
    pub xws: String,

    pub count: u32,

    // just a long string of the sources for informational purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<String>,
}

impl ShipRecord {
    /// Turns skus and xws id's into display names.
    pub fn build(xws: &str, count: u32, data: &Data, catalog: &Catalog) -> Result<Self, ErrorKind> {
        match data.get_ship(xws) {
            None => Err(ErrorKind::NotFound),
            Some(s) => Ok(Self {
                name: s.name.to_owned(),
                xws: s.xws.to_owned(),
                sources: catalog
                    .sources
                    .get(&Item {
                        r#type: ItemType::Ship,
                        xws: xws.to_owned(),
                    })
                    .map(|s| format_sources(catalog, s)),
                count,
            }),
        }
    }
}

/// PilotRecord has fields that I want to sort by so that I can organize my
/// collection, either in binders or boxes.
#[derive(Serialize, Debug)]
pub struct PilotRecord {
    pub faction: String,
    pub ship: String,
    pub xws: String,
    pub name: String,
    pub initiative: u32,

    pub count: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<String>,
}

impl PilotRecord {
    /// Turns skus and xws id's into display names.
    pub fn build(
        xws: &str,
        count: u32,
        data: &Data,
        expansions: &Catalog,
    ) -> Result<Self, ErrorKind> {
        // TODO: there must be a better way to do the restrictions
        match data.get_pilot(xws) {
            None => Err(ErrorKind::NotFound),
            Some((s, p)) => Ok(Self {
                faction: s.faction.to_owned(),
                ship: s.name.to_owned(),
                name: p.name.to_owned(),
                xws: p.xws.to_owned(),
                initiative: p.initiative,
                count,
                sources: expansions
                    .sources
                    .get(&Item {
                        r#type: ItemType::Pilot,
                        xws: xws.to_owned(),
                    })
                    .map(|s| format_sources(expansions, s)),
            }),
        }
    }
}

/// UpgradeRecord are the fields I sort my collection by.
#[derive(Serialize, Debug)]
pub struct UpgradeRecord {
    pub xws: String,
    pub r#type: String,
    pub name: String,
    pub faction_restriction: String,
    pub size_restriction: String,
    pub ship_restriction: String,
    pub arc_restriction: String,
    pub keyword_restriction: String,

    pub count: u32,
    pub force_side_restriction: String,

    // just a long string of the sources for informational purposes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<String>,
}

impl UpgradeRecord {
    /// Turns skus and xws id's into display names.
    pub fn build(xws: &str, count: u32, data: &Data, catalog: &Catalog) -> Result<Self, ErrorKind> {
        // TODO: there must be a better way to do the restrictions
        match data.get_upgrade(xws) {
            None => Err(ErrorKind::NotFound),
            Some(u) => Ok(Self {
                name: u.name.to_owned(),
                xws: u.xws.to_owned(),
                count,
                r#type: u
                    .sides
                    .first()
                    .map(|s| format!("{:?}", s.r#type)) //FIXME
                    .unwrap_or("not found".to_owned())
                    .to_owned(),
                faction_restriction: format_restriction(
                    data,
                    &u.restrictions,
                    Restriction::Factions,
                ),
                size_restriction: format_restriction(data, &u.restrictions, Restriction::Sizes),
                ship_restriction: format_restriction(data, &u.restrictions, Restriction::Ships),
                keyword_restriction: format_restriction(
                    data,
                    &u.restrictions,
                    Restriction::Keywords,
                ),
                force_side_restriction: format_restriction(
                    data,
                    &u.restrictions,
                    Restriction::ForceSide,
                ),
                arc_restriction: format_restriction(data, &u.restrictions, Restriction::Arcs),
                sources: catalog
                    .sources
                    .get(&Item {
                        r#type: ItemType::Upgrade,
                        xws: xws.to_owned(),
                    })
                    .map(|s| format_sources(catalog, s)),
            }),
        }
    }
}

fn format_restriction(
    data: &xwingdata2::Data,
    restrictions: &Vec<xwingdata2::Restrictions>,
    kind: xwingdata2::Restriction,
) -> String {
    // TODO: I'm sure there is more efficient way to append these
    let mut tmp: Vec<&str> = vec![];
    for r in restrictions {
        match kind {
            Restriction::Factions => &r
                .factions
                .iter()
                .map(|xws| {
                    data.get_faction(xws.as_str())
                        .map_or(xws.as_str(), |f| f.name.as_str())
                })
                .for_each(|v| tmp.push(v)),
            Restriction::Ships => &r
                .ships
                .iter()
                .map(|xws| {
                    data.get_ship(xws.as_str())
                        .map_or(xws.as_str(), |s| s.name.as_str())
                })
                .for_each(|v| tmp.push(v)),
            Restriction::Sizes => &r.sizes.iter().for_each(|v| tmp.push(v)),
            Restriction::Arcs => &r.arcs.iter().for_each(|v| tmp.push(v)),
            Restriction::Keywords => &r.keywords.iter().for_each(|v| tmp.push(v)),
            Restriction::ForceSide => &r.force_side.iter().for_each(|v| tmp.push(v)),
        };
    }
    tmp.join(",")
}

fn format_sources(expansions: &expansions::Catalog, sources: &Vec<ItemCount>) -> String {
    let mut strs = vec![];

    for s in sources {
        let (name, wave) = expansions
            .expansions
            .get(&s.item.xws)
            .map_or(("unknown", 99), |e| (&e.name, e.wave));
        strs.push(format!("{}:{}:wave{}:{}", name, s.item.xws, wave, s.count));
    }

    strs.join(",")
}

// TODO: Figure out what is generic here
#[derive(Default, Serialize)]
pub struct Records {
    pub ships: Vec<ShipRecord>,
    pub pilots: Vec<PilotRecord>,
    pub upgrades: Vec<UpgradeRecord>,
}

impl Records {
    pub fn build(inventory: &Inventory, data: &Data, catalog: &Catalog) -> Records {
        let mut records = Records::default();

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
}

pub fn generate_xls(
    catalog: &Catalog,
    data: &Data,
    collection: &Collection,
    inventory: &Inventory,
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    add_expansion_sheet(&mut workbook, catalog, collection)?;
    // This must be done seperately because of the way borrows work on the
    // workbook make it hard to work with more than 1 sheet at once.
    add_ships_sheet(&mut workbook, catalog, data, collection, inventory)?;
    add_pilots_sheet(&mut workbook, catalog, data, collection, inventory)?;
    add_upgrades_sheet(&mut workbook, catalog, data, collection, inventory)?;

    workbook.save("XWingTMG2_Inventory.xlsx")?;

    Ok(())
}

const EXPANSION_COLS: [&str; 4] = ["Owned", "Name", "Wave", "SKU"];

fn add_expansion_sheet(
    workbook: &mut Workbook,
    catalog: &Catalog,
    collection: &Collection,
) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet().set_name("Expansions")?;
    for (i, col) in EXPANSION_COLS.iter().enumerate() {
        worksheet.write(0, i as u16, *col)?;
    }
    let unknown = Expansion {
        sku: "error".to_string(),
        name: "notfound".to_string(),
        wave: 0,
        contents: vec![],
    };
    let mut row = 1;
    for (e, c) in &collection.skus {
        let exp = catalog.expansions.get(e).unwrap_or(&unknown);
        worksheet.write(row, 0, *c)?;
        worksheet.write(row, 1, &exp.name)?;
        worksheet.write(row, 2, exp.wave)?;
        worksheet.write(row, 3, &exp.sku)?;
        row += 1;
    }
    let mut table = Table::new();
    table.set_style(TableStyle::Medium2);
    table.set_name("ExpansionLookup");
    worksheet.add_table(0, 0, row - 1, (EXPANSION_COLS.len() as u16) - 1, &table)?;
    worksheet.autofit();
    Ok(())
}

fn total_func(item: &Item, singles_cell: String, catalog: &Catalog) -> String {
    let mut func = format!("={}", singles_cell);

    if let Some(sources) = catalog.sources.get(item) {
        for source in sources {
            func.push_str(&format!("+{}*XLOOKUP(\"", source.count));
            func.push_str(&source.item.xws);
            func.push_str("\",ExpansionLookup[SKU],ExpansionLookup[Owned],0,0)");
        }
    }

    func
}

const SHIP_COLS: [&str; 5] = ["Name", "Total", "Singles", "XWS", "Sources"];
fn add_ships_sheet(
    workbook: &mut Workbook,
    catalog: &Catalog,
    data: &Data,
    collection: &Collection,
    inventory: &BTreeMap<Item, u32>,
) -> Result<(), XlsxError> {
    let ships = workbook.add_worksheet().set_name("Ships")?;
    for (i, col) in SHIP_COLS.iter().enumerate() {
        ships.write(0, i as u16, *col)?;
    }

    let mut ship_row = 1;
    let ship_singles_col = 2;
    for item in inventory.keys() {
        if item.r#type == ItemType::Ship {
            let model = match data.get_ship(&item.xws) {
                Some(m) => m,
                None => {
                    println!("xslx: missing ship {}", item.xws);
                    continue;
                }
            };

            ships.write(ship_row, 0, &model.name)?;
            ships.write_dynamic_formula(
                ship_row,
                1,
                total_func(item, row_col_to_cell(ship_row, ship_singles_col), catalog).as_str(),
            )?;
            ships.write(
                ship_row,
                2,
                *collection.singles.get(item).unwrap_or(&0) as i32,
            )?;
            ships.write(ship_row, 3, &model.xws)?;
            ships.write(
                ship_row,
                4,
                catalog
                    .sources
                    .get(item)
                    .map(|s| format_sources(catalog, s))
                    .unwrap_or("".to_string()), //.unwrap_or("".to_string()),
            )?;

            ship_row += 1;
        }
    }
    let mut table = Table::new();
    table.set_name("ShipTable");
    table.set_style(TableStyle::Medium3);
    ships.add_table(0, 0, ship_row - 1, SHIP_COLS.len() as u16 - 1, &table)?;
    ships.autofit();
    Ok(())
}

const PILOT_COLS: [&str; 8] = [
    "Name",
    "Ship",
    "Initiative",
    "Faction",
    "Total",
    "Singles",
    "XWS",
    "Sources",
];
fn add_pilots_sheet(
    workbook: &mut Workbook,
    catalog: &Catalog,
    data: &Data,
    collection: &Collection,
    inventory: &BTreeMap<Item, u32>,
) -> Result<(), XlsxError> {
    let pilots = workbook.add_worksheet().set_name("Pilots")?;
    for (i, col) in PILOT_COLS.iter().enumerate() {
        pilots.write(0, i as u16, *col)?;
    }

    let mut pilot_row = 1;
    let pilot_singles_col = 5;
    for item in inventory.keys() {
        if item.r#type == ItemType::Pilot {
            // TODO: probably don't need to
            let (ship, pilot) = match data.get_pilot(&item.xws) {
                Some(m) => m,
                None => {
                    println!("xslx: missing pilot {}", item.xws);
                    continue;
                }
            };

            pilots.write(pilot_row, 0, &pilot.name)?;
            pilots.write(pilot_row, 1, &ship.name)?;
            pilots.write(pilot_row, 2, pilot.initiative)?;
            pilots.write(pilot_row, 3, &ship.faction)?;
            pilots.write_dynamic_formula(
                pilot_row,
                4,
                total_func(item, row_col_to_cell(pilot_row, pilot_singles_col), catalog).as_str(),
            )?;
            pilots.write(
                pilot_row,
                5,
                *collection.singles.get(item).unwrap_or(&0) as i32,
            )?;
            pilots.write(pilot_row, 6, &pilot.xws)?;
            pilots.write(
                pilot_row,
                7,
                catalog
                    .sources
                    .get(item)
                    .map(|s| format_sources(catalog, s))
                    .unwrap_or("".to_string()), //.unwrap_or("".to_string()),
            )?;

            pilot_row += 1;
        }
    }
    let mut table = Table::new();
    table.set_name("pilotTable");
    table.set_style(TableStyle::Medium4);
    pilots.add_table(0, 0, pilot_row - 1, PILOT_COLS.len() as u16 - 1, &table)?;
    pilots.autofit();
    Ok(())
}

const UPGRADE_COLS: [&str; 12] = [
    "Name",
    "Type",
    "Faction Restriction",
    "Ship Restriction",
    "Size Restriction",
    "Arc Restriction",
    "Force Side Restriction",
    "Keyword Restriction",
    "Total",
    "Singles",
    "XWS",
    "Sources",
];

fn add_upgrades_sheet(
    workbook: &mut Workbook,
    catalog: &Catalog,
    data: &Data,
    collection: &Collection,
    inventory: &BTreeMap<Item, u32>,
) -> Result<(), XlsxError> {
    let upgrades = workbook.add_worksheet().set_name("Upgrades")?;
    for (i, col) in UPGRADE_COLS.iter().enumerate() {
        upgrades.write(0, i as u16, *col)?;
    }

    let mut upgrade_row = 1;
    let upgrade_singles_col = 9;
    for item in inventory.keys() {
        if item.r#type == ItemType::Upgrade {
            let upgrade = match data.get_upgrade(&item.xws) {
                Some(m) => m,
                None => {
                    println!("xslx: missing upgrade {}", item.xws);
                    continue;
                }
            };

            let record = UpgradeRecord::build(&item.xws, 1, data, catalog).unwrap();

            upgrades.write(upgrade_row, 0, &upgrade.name)?;
            upgrades.write(upgrade_row, 1, &record.r#type)?;

            upgrades.write(upgrade_row, 2, &record.faction_restriction)?;
            upgrades.write(upgrade_row, 3, &record.ship_restriction)?;
            upgrades.write(upgrade_row, 4, &record.size_restriction)?;
            upgrades.write(upgrade_row, 5, &record.arc_restriction)?;
            upgrades.write(upgrade_row, 6, &record.force_side_restriction)?;
            upgrades.write(upgrade_row, 7, &record.keyword_restriction)?;

            upgrades.write_dynamic_formula(
                upgrade_row,
                8,
                total_func(
                    item,
                    row_col_to_cell(upgrade_row, upgrade_singles_col),
                    catalog,
                )
                .as_str(),
            )?;
            upgrades.write(
                upgrade_row,
                9,
                *collection.singles.get(item).unwrap_or(&0) as i32,
            )?;
            upgrades.write(upgrade_row, 10, &upgrade.xws)?;
            upgrades.write(
                upgrade_row,
                11,
                catalog
                    .sources
                    .get(item)
                    .map(|s| format_sources(catalog, s))
                    .unwrap_or("".to_string()), //.unwrap_or("".to_string()),
            )?;

            upgrade_row += 1;
        }
    }
    let mut table = Table::new();
    table.set_name("upgradeTable");
    table.set_style(TableStyle::Medium5);
    upgrades.add_table(0, 0, upgrade_row - 1, UPGRADE_COLS.len() as u16 - 1, &table)?;
    upgrades.autofit();
    Ok(())
}
