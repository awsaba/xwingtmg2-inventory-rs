# X-Wing: The Miniature Game 2.0/2.5 Inventory Management Tools

This repository generates an Microsoft® Excel® for Microsoft 365 MSO for keeping
track of an X-Wing: The Miniature Game 2.0/2.5 collection.

As a command line interface (CLI), it gives you the ability to take your raw
[yasb collection](https://login.yasb.app/collection) and dump it to a `json`
file that can be further processed with something `jq`.

## Using the produced spreadsheet

1. Download the [latest spreadsheet](https://github.com/awsaba/xwingtmg2-inventory-rs/releases/latest/download/XWingTMG2_Inventory.xlsx)
   from the `Releases`.
1. In the `Expansions` sheet, input the number of each expansion you own in the
   `Owned` column.
1. In the other sheets, `Ships`, `Pilots`, `Upgrades`, add any loose ships, such
   as 1.0 models still in your collection to the `Singles` column.

That's it. The `Totals` column will update with your `Singles` and `Expansion`
counts per-item summed.

## Using the CLI directly with a YASB collection

1. You will need a working `rust` toolchain. Refer to the installation and usage instructions for your platform.
1. Clone this repo and its submodules: `git submodule init`.
1. Log in to <https://yasb.app>, access your raw collection at <https://login.yasb.app/collection>, and save the `json` to `collection.json`.
1. Run the tool with `cargo run`. This will produce an `inventory.json`, which will contain
   `ships`, `pilots`, and `upgrades` lists.
1. Use something like `jq` to turn it into CSV [(from StackOverflow)](https://stackoverflow.com/questions/32960857/how-to-convert-arbitrary-simple-json-to-csv-using-jq):

```shell
cargo run -- --format json --collection collection.json
jq -r '.pilots | (map(keys) | add | unique) as $cols | map(. as $row | $cols | map($row[.])) as $rows | $cols, $rows[] | @csv' inventory.json > pilots.csv
jq -r '.upgrades | (map(keys) | add | unique) as $cols | map(. as $row | $cols | map($row[.])) as $rows | $cols, $rows[] | @csv' inventory.json > upgrades.csv
jq -r '.ships | (map(keys) | add | unique) as $cols | map(. as $row | $cols | map($row[.])) as $rows | $cols, $rows[] | @csv' inventory.json > ships.csv
```

## Why does this exist

1. My collection is far from complete, but I still have over 1100+ pilot and
   upgrade cards that don't fit in a single organizer
   or box, so I need a way to more accurately count by type and restrictions to
   help me organize.
1. How do I get X if I don't have it? What am I actually missing out
   on if I don't pick up an expansion?
1. List building tools are focused on the list building, so inventory management
   is a much lower priority for them.
1. I also couldn't find anyone keeping an updated Excel template or anything else
   of that sort.
1. I like the idea of being able to keep a copy of my inventory list local and
   flexible.
1. Hopefully this is useful to someone else, as either an exercise in working
  `xwing-data2` or as inspiration for something better.

Why rust?

1. I'm not a JavaScript/coffee/typescript programmer and need an excuse
   to use `rust` at all since it is not my day job currently.
2. I tried to use the [pyxwb2](https://pypi.org/project/pyxwb2/) and immediately
   ran into having to do multiple checks for optional items such as `restrictions`,
   and figured I may as well get the compiler's help when working with them.

## Challenges

* XWS has a definitive definition of every item in the game, but `xwing-data2`
  lacks a listing of expansion contents, so
  [expansions.json](src/expansions/expansions.json) will need to continue to be
  updated.
* YASB uses names and not unique IDs in their collections and those change over
  time. YASB is pretty popular, so the goal will be to update to support
  importing from a YASB collection dump.
* Windows and character encodings. *Run the following nonsense in your PowerShell
  session before trying any of this*:

  ```shell
  > [System.Console]::OutputEncoding=[System.Text.Encoding]::UTF8
  ```

## Known Issues

* YASB Collection Quirks
  * If you used YASP pre-2.0, your 1.0 expansions are still there, but of
    won't be in YASB, except for the core sets and the ships that came with
    obstacles, so they will print out as warning.
  * "First Edition VT-49 Decimator": Records appear to have been removed when the VT-49 was
    reprinted, so can't be imported like the other 1.0 obstacle sets.

## Contributing to the this repo

Have a look at the issues.

This is a basic `rust` project with some tests run in GitHub Actions, try to add
some tests for anything you are going to add.

### Adding new expansions

1. Update the `xwing-data2` submodule to a version that includes the expansion
   contents.
1. Add expansion to [src/expansions/expansions.json](src/expansions/expansions.json).
1. TODO
1. Check if any ships can be removed from the `swzunreleased` placeholder.

## Expansions

Wave numbers are based on sku, which is *mostly* related to the release date.
This is in contrast to the [waves in the wiki](https://xwing-miniatures-second-edition.fandom.com/wiki/Products)
which groups by announcement date and separates some expansions which I think
are better grouped by theme.

### Initial `expansions.json`

See [this terrible script](https://github.com/awsaba/xwing/blob/awsaba/xws-content-dumper/coffeescripts/content/dump-content.coffee).

## Acknowledgements and Licenses

All Star Wars and X-Wing: TMG content itself is copyright and trademark of its
owners: FFG/AMG/LucasArts.

The initial content for `expansions.json` and any `yasb` related conversions
are from the [`yasb`](https://github.com/raithos) project, copyright
2012 Geordan Rosario and others uses under an MIT style license.

This also would not be possible with [`xwing-data2`], also under an MIT license.

Other content fixes are from the [X-Wing Miniatures: Second Edition Wiki](https://xwing-miniatures-second-edition.fandom.com/wiki/X-Wing_Miniatures:_Second_Edition_Wiki),
community content is covered by CC BY-SA.

[`xwing-data2`]: https://github.com/guidokessels/xwing-data2
