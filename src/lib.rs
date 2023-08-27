use serde::Serialize;

pub mod expansions;
pub mod xwingdata2;
pub mod yasb2;

use xwingdata2::Restriction;

/// This is the full ship as defined by the expansions.
///
/// TODO: Add a "miniature/chassis" type that reflects usability per tournament
/// regulations.
#[derive(Serialize, Debug)]
pub struct ShipRecord {
    pub name: String,
    pub xws: String,

    pub count: u32,
}

impl ShipRecord {
    pub fn new(s: &xwingdata2::Ship, count: u32) -> ShipRecord {
        ShipRecord {
            name: s.name.to_owned(),
            xws: s.xws.to_owned(),
            count,
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
}

/// UpgradeRecord are the fields I sort my collection by.
#[derive(Serialize, Debug)]
pub struct UpgradeRecord {
    pub r#type: String,
    pub name: String,
    pub faction_restriction: String,
    pub size_restriction: String,
    pub ship_restriction: String,
    pub arc_restriction: String,
    pub keyword_restriction: String,

    pub count: u32,
    pub force_side_restriction: String,

    pub xws: String,
}

impl UpgradeRecord {
    pub fn new(u: &xwingdata2::Upgrade, c: u32) -> UpgradeRecord {
        // TODO: there must be a better way

        UpgradeRecord {
            name: u.name.to_owned(),
            xws: u.xws.to_owned(),
            count: c,
            r#type: u
                .sides
                .first()
                .map(|s| s.r#type.to_owned())
                .unwrap_or("not found".to_owned())
                .to_owned(),
            faction_restriction: format_restriction(&u.restrictions, Restriction::Factions),
            size_restriction: format_restriction(&u.restrictions, Restriction::Sizes),
            ship_restriction: format_restriction(&u.restrictions, Restriction::Ships),
            keyword_restriction: format_restriction(&u.restrictions, Restriction::Keywords),
            force_side_restriction: format_restriction(&u.restrictions, Restriction::ForceSide),
            arc_restriction: format_restriction(&u.restrictions, Restriction::Arcs),
        }
    }
}

fn format_restriction(
    restrictions: &Vec<xwingdata2::Restrictions>,
    kind: xwingdata2::Restriction,
) -> String {
    // TODO: I'm sure there is more efficient way to append these
    let mut tmp: Vec<String> = vec![];
    for r in restrictions {
        let criteria = match kind {
            Restriction::Factions => &r.factions,
            Restriction::Sizes => &r.sizes,
            Restriction::Ships => &r.ships,
            Restriction::Arcs => &r.arcs,
            Restriction::Keywords => &r.keywords,
            Restriction::ForceSide => &r.force_side,
        };
        if !criteria.is_empty() {
            tmp.push(criteria.join(","))
        }
    }
    tmp.join(",")
}
