# modlem: A Lemmings Graphics Editor

## What is modlem?

modlem is a tool to extract and import graphics from the MS-DOS versions of the
game Lemmings™, made by DMA Design and published by Psygnosis.

At present, modlem can:

* Decompress and compress Lemmings' PowerPacker-inspired .DAT file format.
* Extract VGA graphics sets.
* Recreate VGA graphics sets.
* Extract the VGA and 'High-Performance PC' menu graphics.
* Recreate 'main.dat' with edited VGA graphics

It _cannot_:

* Extract the EGA, CGA, or TGA graphics.
* Edit Oh-No! More Lemmings! (without some minor tweaks)
* Extract or edit the Lemmings for Windows files.
* Edit levels (though they can be extracted from .DAT files)
* Edit any palettes other than the graphics set ones.
* Edit any sound effects or music (though they can be extracted from .DAT files)

Some of these are inherent limitations, some may arrive in a future version.

**Warning:** Not all possible images can be successfully compressed — if an image
is at any point larger compressed than uncompressed, Lemmings will fail to load
is successfully. This is a limitation of the game, but it is unlikely to appear
with most images, unless you're going for random noise. :-)

Also note, that this started as a very early experiment with Rust, so the code
is horrible. I may try to tidy it up one day, should I have time.

## How do I use it?

modlem provides several commands:

#### extract-dat: Extract a .dat file

In Lemmings, most data is stored in a compressed .dat archive: these archives
contain several sections, which are numbered. In modlem, you can extract these
sections to individual files with:

```
modlem extract-dat [name]
```

(Where ``[name]`` is the base name of the dat file, for example, ``main`` for
``main.dat``.)

The resulting sections will be saved as numbered files, e.g. main.000,
main.001, etc.

You can re-assemble the dat file with:

#### create-dat: Create a .dat file from numbered sections

This is effectively the reverse of extract-dat above. It creates \[name].dat
from the files \[name].000, \[name].001, etc.

Usage:

```
modlem create-dat [name]
```

To edit the contents of a dat file, you can use extract-dat, modify or replace
the individual section files, then run create-dat to reconstitute it.

This is most useful for editing levels: the level00?.dat files each contain
several levels, each in their own section in the .lvl file format used by
LemEdit and Windows Lemmings.

#### extract-set: Extract a Graphics Set / Theme

Lemmings levels each use a "theme" (also known as a "style" or a "graphics
set"). This command will extract all of the graphics and other miscellaneous
data needed to create a theme.

Themes are stored in two files, ground?o.dat, which contains information about
the graphics, and vgagr?.dat, which is a .dat archive containing the actual
pixel data (at least, for the EGA/VGA version). modlem can extract a graphics
set with:

modlem extract-set \[n]

Where \[n] is the number of the graphics set (from 0–4 in the original
Lemmings).

modlem produces a large number of files as a result. The main one is called
theme\[n].txt, and contains a script listing all of the non-graphics data, and
the filenames of all of the graphics (as windows .bmp files).

Within this script are several commands:

- HeaderFile \[filename] — contains the \[filename] to store the header data
  in, usually something like ground0o.dat
- DataFile \[filename] — contains the \[filename] to store pixel data in,
  usually vgagr0.dat
- Terrain \[filename] — contains the \[filename] of the next bit of terrain in
  the set. May also have a "Mask \[filename]" command giving a second bitmap
  containing mask/transparency data.
- Object \[filename] — contains the \[filename] of an animated object (this
  file contains all of the frames, in a filmstrip format). Also followed by a
  "Mask \[filename]" directive with mask/transparency data, and a number of
  additional options:
  - animation_flags: a number representing animation options for the object
  - frames: the start and end frames of the animation used when the object is
    active
  - trigger: the coordinates of the top-left and bottom-right corners of the
    object's activation rectangle. If a lemming touches this, the object is
    activated.
  - trigger_effect: an effect number describing what the object does when
    activated
  - preview_frame: the frame of animation used for the object in the level
    preview
  - trap_sound: a sound number played when the trap is activated
- Palettes — a list of palettes in EGA or VGA format, as RGB triplets. EGA
  palettes are 2 bit per channel (take values 0–3), VGA palettes (used in the
  extracted bitmaps) are 6 bit per channel (take values 0–63).

The generated theme\[n].txt and corresponding bitmaps can be reconstituted into
a graphics set (the HeaderFile and DataFile) using:

#### create-set: Reconsitute a Graphics Set / Theme

The create-set option rebuilds the ``ground?o.dat`` and ``vgagr?.dat`` files
from the theme script file (e.g. ``theme1.txt``) and the associated bitmaps.

Note that palette data is read from the ``Palettes`` section in the theme script,
not from the bitmap files.

For example:
```
modlem create-set theme0.txt
```

#### extract-main: Extract the data from main.dat

Most of the remaining graphics, including the menu images, lemming sprites, and
fonts live in the ``main.dat`` file. (In addition to the PC-speaker sound
effects, for some reason.)

These graphics can be extracted (and edited) using modlem (though you'll just
get binary data for the PC-speaker effects). Note that the sizes of the images
(and the palettes) are _hardcoded_, and cannot be changed.

The extracted data comes in several bitmaps, including:

- ``lemming_*.bmp``: the individual lemming animations. One file per skill and
  direction. (e.g., ``lemming_brolly_r.bmp`` for a lemming carrying an umbrella,
  facing right.
- ``mask_*.bmp`` : Terrain masks used by skills. These are the shapes removed
  from the level when certain skills are used. And the bomber countdown timer,
  for some reason.
- ``interface_hi_*.bmp``: The interface for the _High-Performance PCs mode``,
  which uses a different palette. Not used in the Christmas Lemmings versions,
  as far as I can tell.
- ``menu_*.bmp``: The graphics for the main menu.
- ``menuanim_*.bmp``: Animations on the main menu (and the difficulty selection).
- ``pcspkr.snd``: The raw PC-Speaker sound effects. If you work out how to edit
  these, let me know!
- ``interface_lo_*.bmp``: The normal interface (for non-"High-Performance" PCs).

You can extract these with:

```
modlem extract-main
```

Or, for Christmas Lemmings,

```
modlem extract-main --christmas
```

You can then recreate them with:

#### create-main: Create a main.dat from bitmap files

This is the opposite of ``extract-main`` above, and will generate a ``main.dat``
file from the bitmap files (and ``pcspkr.snd``) listed above. As the palette
is hardcoded, the bitmaps' palettes are ignored. Similarly, the sizes of the
images (and number of frames of the animations) are all hardcoded, and cannot
change.

Note, also, that some of the images only contain some colours. The game only
stores enough planes for the colours used, so, for example, only four colours
(black/transparent, blue, green, and white) can be used for most of the lemmings
animations.

Usage:

```
modlem create-main
```

### A note on case-sensitivity

As DOS is case-insensitive, modlem makes a half-hearted effort to detect and use
the original game's files even if their filename doesn't directly match on a
case-sensitive filesystem.

The files that modlem produces itself (including the theme script files and
bitmap files) are used as-is, and must match on a case-sensitive filesystem.

But when in doubt, make all of the files lowercase.

## Compiling from Source

If you'd rather compile modlem yourself, you'll need a Rust compiler for your
target platform. 

To build with cargo, simply run:

```
cargo build
```

Or you can run it directly (using the arguments described above), with, for
example,

cargo run extract-dat

Alternatively, if you'd rather not use cargo, you can build it directly with
the included makefile:

```
make
```

## Credits and Acknowledgements

Special thanks to:

* DMA Design, and in particular Mike Daily and Russel Kay, for making these
  games and file formats in the first place.
* ccexplore, for all of his excellent file format documentation
* rt for the `lvl` file description.
* Mindless, for the `main.dat` description
* VTM Software, for the old CustLem / LemEdit tools.
* Nico François, for developing the 'PowerPacker' compresor, which clearly
  inspired the `.DAT` compression.

And see also, [Digger Decoder](https://github.com/chrishulbert/digger-decoder),
a similar tool, the release of which convinced me to dig this up and upload it
somewhere.
