ReCoF Launcher, bundled artwork
===============================

This folder is a slot. The launcher draws its own look when it is
empty, so a clone with nothing in here still builds and still runs.

What is in here now:

  hero.jpg    the background of the main screen, by Greez, used with
              permission. See CREDITS.md at the root of the repository.

What else it takes, with no code change anywhere:

  logo.*      the title drawn over the background
  icon.*      the window icon

Any of png, jpg or jpeg. Photographs belong in a JPEG and anything that
needs transparency belongs in a PNG, so the slot takes both.

Then write CREDITS.txt next to them, one line per picture, naming who
made each one. The launcher shows it in Settings under Appearance and
in About.

Where each picture ends up, in order:

  1. whatever the user put in %LOCALAPPDATA%\ReCoF Launcher\theme
  2. whatever is in this folder, built into the executable
  3. what the launcher draws itself

So a user can always override what ships, and a build with an empty
folder always works.

Before you put anything here
----------------------------

Committing a file here publishes it: it goes into the repository, into
every clone, and into the executable that every release hands out. That
is a decision about somebody else's work, and it belongs to whoever
makes this repository, not to the code.

If you did not make a picture and do not have permission to publish it,
do not put it here. The theme folder in point 1 above exists for exactly
that case: it keeps a picture on one machine, where putting it is
nobody's business but the person who put it there.

If you did get permission, write it down. Every line in CREDITS.txt has
a matching entry in CREDITS.md at the root, with the evidence in
docs/permissions. That is what makes the credit checkable by somebody
who was not there when it was given.
