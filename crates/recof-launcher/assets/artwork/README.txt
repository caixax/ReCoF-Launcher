ReCoF Launcher, bundled artwork
===============================

This folder is a slot. It ships empty, and the launcher draws its own
look when it is empty.

Put any of these here and the next build embeds them, with no code
change anywhere:

  hero.png    the background of the main screen
  logo.png    the title drawn over it
  icon.png    the window icon

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
leave this folder empty. The theme folder in point 1 above exists for
exactly that case: it keeps a picture on one machine, where putting it
is nobody's business but the person who put it there.
