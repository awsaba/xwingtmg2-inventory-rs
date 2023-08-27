# yasb collection to CSV dumper

This is tool that will take your raw [yasb collection](https://yasb.app/collection)
and dump it as comma separated value (CSV) files that can then be processed
in your favorite spreadsheet program.

## Why does this exist

1. List building tools are focused on the list building, so inventory management
   is a much lower priority for them.
1. I also couldn't find anyone keeping an updated Excel template or anything else
   of that sort.
1. I have over 800 pilot and upgrade cards that don't fit in a single organizer
   or box, so I need a way to more accurately count by type and restrictions to
   help me organize.
1. I like the idea of being able to keep a copy of my inventory list local and
   flexible.
1. Hopefully this is useful, as either an exercise in working `xwing-data2` or
   as inspiration for something better.

Why rust?

1. I'm not a JavaScript/coffee/typescript programmer and need an excuse
   to use an `rust` at all since it is not my day job currently.
2. I tried to use the [pyxwb2](https://pypi.org/project/pyxwb2/) and immediately
   ran into having to do multiple checks for optional items such as `restrictions`.

## Challenges

XWS has a definitive definition of every item in the game, but `xwing-data2`
lacks a listing of expansion contents, so [expansions.json](src/expansions.json)
will need to continue to be updated.

YASB renames

YASB is pretty popular, so the goal will be to remain consistent with their
expansion naming for collections.

Windows and character encodings. *Run the following nonsense in your PowerShell
session before trying any of this*:

```shell
> [System.Console]::OutputEncoding=[System.Text.Encoding]::UTF8
```

## Getting your collection

Log in to <https://yasb.app>, access your collection at <https://yasb.app/collection>,
   and save the `json` to `collection.json`.

> **Note**
> YASB has renamed ships/expansions over time, so if you haven't
> "Reset my collection" in a while, you may end up with a collection that double
> counts items.

## Using this tool

1. You will need a working rust toolchain.
1. Clone this repo and it's submodules: `git submodule init`
   1. Note: You may need to update to a more current version of [`xwing-data2`]
1. Run the tool with `cargo run`. This will product 2 `json` files, one for pilots
   and one for upgrades.
1. Use something like `jq` to turn it into CSV [(from StackOverflow)](https://stackoverflow.com/questions/32960857/how-to-convert-arbitrary-simple-json-to-csv-using-jq):

```shell
jq -r '(map(keys) | add | unique) as $cols | map(. as $row | $cols | map($row[.])) as $rows | $cols, $rows[] | @csv' pilots.json > pilots.csv
jq -r '(map(keys) | add | unique) as $cols | map(. as $row | $cols | map($row[.])) as $rows | $cols, $rows[] | @csv' upgrades.json > upgrades.csv
```

## Contributing to the this repo

Have a look at the issues.

This is a basic `rust` project with some tests run in GitHub Actions, try to add
some tests for anything you are going to add.

### Adding new expansions

1. Add expansion.
1. ...
1. Check if any ships can be removed from the `looseships` placeholder.

## Initial `expansions.json`

This is what I did because I couldn't get the toolchain for `raithos/xwing`
installed correctly.

1. Prerequisite: at least `node` can be run.
1. Cloned `raithos/xwing`.
1. Added it to VS Code.
1. Installed "CoffeeScript Preview" extension.
1. Previewed `coffeescript/content/manifest.coffee`, save the produced JavaScript file.
1. Add the following to generated file:

   ```javascript
     util = require('util')
     console.log(JSON.stringify(exportObj.manifestByExpansion, null, 2))
   ```

1. Pipe to a file: `node .\manifest.coffee.js > expansions.json`

## Acknowledgements and Licenses

All Star Wars and X-Wing: TMG content itself is copyright and trademark of its
owners: FFG/AMG/LucasArts.

The initial content for `expansions.json` and any `yasb` related conversions
are from the [`yasb`](https://github.com/raithos) project, copyright
2012 Geordan Rosario and others uses under an MIT style license.

This also would not be possible with [`xwing-data2`], also under an MIT license.

Other content fixes are from the [X-Wing Miniatures: Second Edition Wiki](https://xwing-miniatures-second-edition.fandom.com/wiki/X-Wing_Miniatures:_Second_Edition_Wiki),
community content is covered by CC BY-SA.

[`xwing-data2`](https://github.com/guidokessels/xwing-data2),
