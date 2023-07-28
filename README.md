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
lacks a listing of expansion contents, so that will continue to be updated.

The initial list of expansion contents was pulled from

YASB is pretty popular, so the goal will be to remain consistent with their
expansion naming for collections.

Windows and character encodings. *Run the following nonsense in your PowerShell
session before trying any of this*:

```shell
> [System.Console]::OutputEncoding=[System.Text.Encoding]::UTF8
```

## Using this tool

1. You will need a working rust toolchain.
1. Clone the `xwing-data2` repository. (TODO: add as submodule?)
1. Log in to <https://yasb.app>, access your collection at <https://yasb.app/collection>,
   and save the `json` to `collection.json`.
1. Run the tool with `cargo run`. This will product 2 `json` files, one for pilots
   and one for upgrades.
1. Use something like `jq` to turn it into CSV [(from StackOverflow)](https://stackoverflow.com/questions/32960857/how-to-convert-arbitrary-simple-json-to-csv-using-jq):

```shell
cat pilots.json | jq -r '(map(keys) | add | unique) as $cols | map(. as $row | $cols | map($row[.])) as $rows | $cols, $rows[] | @csv'
```

## TODOs and other thoughts

Contributions welcome!

- Optionally output direct to CSV.
- Remove `unwrap` and other `rust` no-nos.
- Add support ships and dials. Like, I have lost track of my T-65s and
  valid dials I have for them.
- Tests: Add at least 1 of everything to a collection in YASB and use that as
  input.
- GitHub actions for linting, building, and testing.
- Embed necessary data and build a binary.
- Generate a lookup from the YASB names to the `xws` id using the yasb card listing.
- Make expansions support separate from yasb. Working with yasb requires looking up
  from the YASB expansion and card names to something like XWS name in any case,
  so this would help to support collection lists from other builders.
- Use something like [typify](https://github.com/oxidecomputer/typify) to
  generate a full schema for `xws`. I hand wrote the types for importing needed
  for my need: tell me how many card organizer pages I need for an upgrade
  type/faction/restriction.

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
