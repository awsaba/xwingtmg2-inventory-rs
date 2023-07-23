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

Why rust?

1. I'm not a javascript/coffeescript/typescript programmer and need an excuse
   to use an `rust` at all since it is not my day job currently.
2. I tried to use the [pyxwb2](https://pypi.org/project/pyxwb2/) and immediately
   ran into having to do multiple checks for optional items such as `restrictions`.

## Challenges

XWS has a definitive definition of every item in the game, but `xwing-data2`
lacks a listing of expansion contents, so that will continue to be updated.

The initial list of expansion contents was pulled from

YASB is pretty popular, so the goal will be to remain consistent with their
expansion naming for collections.

## Using this tool

1. You will need a working rust toolchain.
1. Clone the `xwing-data2` repository. (TODO: add as submodule?)
1. Log in to <yasb.app>, access your collection at <https://yasb.app/collection>,
   and save the json to `collection.json`.
1. Run the tool with `cargo run`. This will product 2 csv files, one for pilots
   and one for upgrades.

## TODOs and other thoughts

Contributions welcome!

- Using something like [typify](https://github.com/oxidecomputer/typify) to
  generate a full schema for xws. I had hand wrote the types for importing just meet
  my need: tell me how many card organizer pages I need for an upgrade
  type/faction/restriction.
- Automate the addition of new Expansions somehow? Expansions don't seem to
  coming in fast, so adding one every couple months isn't likely to be a hassle.
