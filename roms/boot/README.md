https://gbdev.gg8.se/files/roms/bootroms/readme.txt

This is a collection of all known boot ROMs dumped from various Gameboy models.
These is the code that is responsible for displaying the scrolling Nintendo
logo on startup, playing the iconic po-ling sound, as well as verifying the
logo and header checksum in an attempt to lock out unlicensed cartridges.

They can be used in supported emulators to emulate the full boot process.

Normally, these ROMs are disabled when control is handed over to the game
cartridge, which makes dumping them difficult, but various methods have been
devised throughout the years to make dumping them possible.

The bootstraps are discussed further on the following GB Dev Wiki page:

https://gbdev.gg8.se/wiki/articles/Gameboy_Bootstrap_ROM

File list
=========
dmg_boot.bin         - The DMG boot ROM.
This is the most common version of the boot ROM found in the original DMG-01
model of Gameboy. 

dmg0_boot.bin        - Early DMG boot ROM with different behavior.
This is the very early variant of the DMG boot ROM which is only found in very
early Japan-sold DMG units. It has a different behavior in that it makes the
screen flash if the boot process fails, instead of scrolling down a 
(potentially corrupted looking) logo and hanging.

mgb_boot.bin         - The Pocket Gameboy boot ROM.
Only differs from dmg_boot.bin in one byte, which loads the value $FF into the
A register instead $01 just before handing over the control to the game. This
can be used by the game to detect that it is running on MGB hardware.

sgb_boot.bin         - The Super Gameboy boot ROM.
Instead of showing a logo animation, this ROM sends the ROM header to the SNES
part of the Super Gameboy, which shows a fancy animation before displaying the
Gameboy screen.

sgb2_boot.bin        - The Super Gameboy 2 boot ROM.
Analogous to mgb_boot.bin, the SGB2 boot ROM only differs from the SGB boot
ROM in one byte loading a value into the A register which allows SGB2 hardware
to be distinguished from SGB hardware.

cgb_boot.bin         - The Gameboy Color boot ROM
The GBC boot ROM is spread out over a bigger memory area than the original 256
bytes, and has slightly increased functionality compared previous boot ROMs,
including the ability to choose a palette from a number of presets for non-GBC
enabled games.

cgb0_boot.bin        - Early Gameboy Color boot ROM
This early revision of the GBC boot ROM was dumped by Matt Currie in 2019 from
a CPU CGB (CPU with no suffix). It has the following differences from the
later more common version of the ROM:
- CGB0 does not initialize Wave RAM, newer revisions do.
- CGB0 uses less optimized code to load the Game Boy logo.
- CGB0 has two redundant writes to RAM, which were removed in newer revisions.

cgb_agb_boot.bin     - Gameboy Color boot ROM used in GBA's GBC mode
This revision of the GBC boot ROM was used in GBA's GBC compatibility mode and
has the following changes from the most common GBC boot ROM revision:
- CGB-AGB copies the Nintendo logo in the cartridge header to a safe place in
  HRAM and uses this copy to both confirm the validity of the logo, and
  render the graphic displayed on the screen. This finally closes the last
  loophole for logo swapping techniques. (It still only checks half the logo
  like previous GBC revisions though.)
- CGB-AGB contains an additional "inc B" instruction right before control is
  handed over to the cartridge, which allows the game to detect that it's 
  running on GBA and for example fix its color palette to improve visibility
  on the darker GBA LCD screen. This also leads to a minor reorganization of
  the code in this area.

gamefighter_boot.bin - Boot ROM from the Gameboy clone Game Fighter.
fortune_boot.bin     - Boot ROM from the Gameboy clone Fortune/Bitman 3000B.
These boot ROMs are dumped from two unlicensed Gameboy clones which each has a
rather different boot ROM compared to the originals.

maxstation_boot.bin  - Boot ROM from the Gameboy clone Maxstation.
This boot ROM is a modified version of the DMG boot ROM, with the following
changes:
- The Maxstation boot ROM copies a "Loading..." graphic from the boot ROM
  instead of the logo found in the cartridge header.
- It copies null bytes to the tile where the (R) symbol would be.
- It has patched out the logo and checksum checks, so cartridges with invalid
  logo/checksum will run.
Because this boot ROM is a patched version of the DMG boot ROM, it follows the
exact same execution patch. This makes it undetectable using CPU registers or
timing, but you can detect it easily form software by looking for the
"Loading..." graphic in tile VRAM.

Using these files with emulators
================================
* BGB
BGB has support for boot ROMs, which can be enabled by going to the system tab
in the settings, entering a path to the ROM in question, and checking "bootroms
enabled".

* Higan
Higan needs the SGB and/or SGB2 boot ROM for correctly running Super Gameboy
enhanced software. The SGB boot ROM in particular can't be emulated through HLE
(high level emulation) since it communicates with the SNES. Using these files
with Higan is documented here:

https://higan.readthedocs.io/en/stable/guides/import/#super-game-boy-games

* Mooneye-GB
Mooneye-GB, an accuracy focused Gameboy emulator written by gekkio (the same
person mentioned in the history section) supports boot ROMs and in fact 
requires them to guarantee cycle accurate emulation.

History
=======
The first time anyone published a dump of such a ROM was in 2003 when neviksti
decapped a DMG CPU and manually read out each individual bit, all 2048 of them.

https://dot-matrix-game.blogspot.com/2014/01/boot-roms.html
https://www.neviksti.com/DMG/

In 2009, the Super Gameboy and Gameboy Color boot ROMs were dumped by Costis
Sideris. The SGB boot ROM was dumped using an overclock attack, whereas the GBC
boot ROM was dumped using a power and clock glitch attack.

https://www.its.caltech.edu/~costis/sgb_hack/

In 2014, BennVenn came up with a simple clock glitching method which requires
nothing but a piece of wire. The method consists of connecting one side of the
crystal oscillator circuit to ground briefly, which makes the CPU jump to a
random place in memory without disabling the boot ROM, which allows it to be
read out. By using this method, he was able to dump the MGB (Gameboy Pocket)
boot ROM.

https://web.archive.org/web/20151014210143/http%3A//www.bennvenn.com/MGB.htm

The same year nitro2k01 (the person running this site and writing this text)
dumped the boot ROM of two unlicensed Gameboy clones using BennVenn's method.
They have a slightly different behavior from the original ones and may be
partially or fully rewritten.

https://blog.gg8.se/wordpress/2014/12/09/dumping-the-boot-rom-of-the-gameboy-clone-game-fighter/

In 2015 gekkio, using another overclock attack, dumped the SGB2 boot ROM.

https://gekkio.fi/blog/2015-09-13-dumping-the-super-game-boy-2-boot-rom.html

In 2016, gekkio also dumped an variant of the DMG boot ROM only found in very
early DMG units, now dubbed dmg0 in the world of Gameboy research.

https://gekkio.fi/blog/2016-10-04-game-boy-research-status.html

File hashes
===========
SHA256:
3a307a41689bee99a9a32ea021bf45136906c86b2e4f06c806738398e4f92e45  cgb0_boot.bin
b4f2e416a35eef52cba161b159c7c8523a92594facb924b3ede0d722867c50c7  cgb_boot.bin
fe3cceb79930c4cb6c6f62f742c2562fd4c96b827584ef8ea89d49b387bd6860  cgb_agb_boot.bin
26e71cf01e301e5dc40e987cd2ecbf6d0276245890ac829db2a25323da86818e  dmg0_boot.bin
cf053eccb4ccafff9e67339d4e78e98dce7d1ed59be819d2a1ba2232c6fce1c7  dmg_boot.bin
9e328227920e86d5530f54efedb562e9ce5b6d32a4ecdee0a278a3d9c6a114b1  fortune_boot.bin
7abdaeea7ac2afd39d86a2ddf044fb978ccd4e65fa4ef15ffc8fcd19df71f254  gamefighter_boot.bin
27e4bee8a8fddc80d48393a51fd9cdf33abc981a795f6aecc59a03a12daff881  maxstation_boot.bin
a8cb5f4f1f16f2573ed2ecd8daedb9c5d1dd2c30a481f9b179b5d725d95eafe2  mgb_boot.bin
fd243c4fb27008986316ce3df29e9cfbcdc0cd52704970555a8bb76edbec3988  sgb2_boot.bin
0e4ddff32fc9d1eeaae812a157dd246459b00c9e14f2f61751f661f32361e360  sgb_boot.bin


SHA1:
df5a0d2d49de38fbd31cc2aab8e62c8550e655c0  cgb0_boot.bin
1293d68bf9643bc4f36954c1e80e38f39864528d  cgb_boot.bin
fa5287e24b0fa533b3b5ef2b28a81245346c1a0f  cgb_agb_boot.bin
8bd501e31921e9601788316dbd3ce9833a97bcbc  dmg0_boot.bin
4ed31ec6b0b175bb109c0eb5fd3d193da823339f  dmg_boot.bin
f9d63ac153c378145fe04c052951ad5cf12ac916  fortune_boot.bin
a4a36f71bf1b3b587df620d48ae940af93a982a5  gamefighter_boot.bin
1776bd61b8db71fc4c4d4b5feab4a21b3c1fd95b  maxstation_boot.bin
4e68f9da03c310e84c523654b9026e51f26ce7f0  mgb_boot.bin
93407ea10d2f30ab96a314d8eca44fe160aea734  sgb2_boot.bin
aa2f50a77dfb4823da96ba99309085a3c6278515  sgb_boot.bin


MD5:
7c773f3c0b01cb73bca8e83227287b7f  cgb0_boot.bin
dbfce9db9deaa2567f6a84fde55f9680  cgb_boot.bin
e6cefb5f7d352fab6681989763917c73  cgb_agb_boot.bin
a8f84a0ac44da5d3f0ee19f9cea80a8c  dmg0_boot.bin
32fbbd84168d3482956eb3c5051637f5  dmg_boot.bin
92ed4eca17d61fcd53f8a64c3ce84743  fortune_boot.bin
6a7b8ee12a793f66a969c6a2b8926cc9  gamefighter_boot.bin
77a7021db824010a678791f6d062943d  maxstation_boot.bin
71a378e71ff30b2d8a1f02bf5c7896aa  mgb_boot.bin
e0430bca9925fb9882148fd2dc2418c1  sgb2_boot.bin
d574d4f9c12f305074798f54c091a8b4  sgb_boot.bin