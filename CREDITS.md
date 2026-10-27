# Credits

ReCoF Launcher draws its own look: the gradient, the vignette, the grain and the type are generated at run time, and the launcher works with no artwork at all. What follows is the artwork it does carry, and the permission behind each piece.

## Artwork

Launcher background art by Greez (https://www.steamgriddb.com/hero/2549), used with permission.

Permission was asked for directly and given directly. The exchange is recorded in [docs/permissions/greez-hero-2549.png](docs/permissions/greez-hero-2549.png).

The file in the repository is `crates/recof-launcher/assets/artwork/hero.jpg`. It is the same picture, resized from 3840x1240 to 2560x827 and saved as a JPEG so it does not dominate the size of the executable. The metadata the original carried was dropped in the process. The original is the one linked above.

The permission covers Greez's work. The picture is built on imagery from Cry of Fear, which belongs to Team Psykskallar, and nothing here is a grant from them. If they ask for it to go, it goes: delete the file from `assets/artwork`, and the next build falls back to the look the launcher draws for itself, with no other change needed.

## Font

The display face is Archivo Black by Omnibus Type, under the SIL Open Font License 1.1. The licence travels with it in `crates/recof-launcher/assets/ArchivoBlack-OFL.txt`.

## Everything else

ReCoF Launcher is MIT licensed and is the work of caixax. It is a separate project from ReCoF itself, and it has no relationship with Team Psykskallar, with Valve or with `CoFLaunchApp.exe`.

No retail asset or binary is distributed here. The base game always comes from the user's own copy.

## Adding to this file

If you put a picture in `crates/recof-launcher/assets/artwork`, add its line to this file, add its line to `CREDITS.txt` beside it so it reaches the screen, and put the permission in `docs/permissions`. A test fails if the folder holds a picture that nobody is credited for.
