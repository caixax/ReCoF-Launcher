# Ideas not implemented

Things that were considered and deliberately left out, with the reason and
what would have to be true before they are worth doing.

## Saving disk space with hardlinks

A side by side install costs a full copy of the game, about 4.4 GiB. Most of
that is read only assets that the copy and the Steam install would never both
write. On the same NTFS volume, hardlinking those files would bring the cost
of a copy close to zero.

Why it is not implemented:

- Every file the game writes must be a real copy, not a link: `config.cfg`,
  `scriptsettings.dat`, `alt.dat`, everything under `SAVE`, `custom.hpk` and
  `steam_appid.txt`. Missing one would mean the game writing into the Steam
  installation through the link, which breaks the first rule of this project.
- Whether a Steam update writes through a hardlink or replaces the file is
  not verified. If it writes through, a game update would corrupt the ReCoF
  copy, or worse, the ReCoF copy would corrupt the Steam install.
- It only works when the copy is on the same volume as the Steam library.

Before implementing: verify Steam's update behaviour against a hardlinked
file, and make the list of writable paths exhaustive and tested. It would
then be an opt in checkbox, never the default.

## Installing over the Steam folder

Rejected. Steam's verify would put the retail binaries back silently, and the
Steam installation must stay untouched. See
`docs/feasibility/launcher.md` in the ReCoF repository for the full
comparison.
