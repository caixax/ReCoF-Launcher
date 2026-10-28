# ReCoF Launcher, English. This is the source language: every other file
# is a translation of this one, and the tests check that they define the
# same keys and interpolate the same values.
#
# Style: plain sentences, no em dashes and no en dashes anywhere.

app-name = ReCoF Launcher
app-tagline = Installer and launcher for ReCoF

## Buttons and words that appear everywhere

button-next = Next
button-back = Back
button-cancel = Cancel
button-close = Close
button-continue = Continue
button-ok = OK
button-yes = Yes
button-no = No
button-browse = Browse...
button-retry = Try again
button-copy = Copy
button-open-folder = Open folder
button-finish = Finish
button-skip = Skip
button-details = Details
word-yes = yes
word-no = no
word-unknown = unknown

## First run: language

firstrun-language-title = Choose your language
firstrun-language-help = You can change this later in settings.
language-not-reviewed = First draft, not yet checked by a native speaker
language-not-reviewed-notice = This translation is a first draft and has not been checked by a native speaker yet. Corrections are welcome.

## First run: what this is

intro-1-title = What ReCoF is
intro-1-body = ReCoF is an improved recompilation of Cry of Fear. This launcher installs it, keeps it updated and starts it. ReCoF and this launcher are separate projects from the game itself.
intro-2-title = Your Steam copy is never touched
intro-2-body = The launcher reads your Steam installation and copies it to a folder you choose. It never writes, deletes or renames anything inside the Steam folder, so Cry of Fear keeps working exactly as it does now and you can go back to it at any time.
intro-3-title = A separate copy, and your progress
intro-3-body = Because ReCoF lives in its own copy, it needs room for the whole game, about { $size }. Your saves, settings and unlocks can be brought over from the Steam copy during the install, or later from the menu. Nothing is ever copied back the other way.

## Finding the game

detect-title = Finding Cry of Fear
detect-searching = Looking for your Steam installation...
detect-found = Found at { $path }
detect-not-found = Cry of Fear was not found
detect-not-found-help = Steam may not be installed, or the game may be in a folder the launcher cannot see. Choose the folder that holds cof.exe.
detect-pick-folder = Choose the game folder
detect-pick-folder-help = The folder that holds cof.exe
detect-library = Steam library: { $path }
detect-buildid-ok = Build { $buildid }, the one ReCoF is built for
detect-buildid-wrong = This is build { $found }. ReCoF is built for build { $expected }. It may not work correctly, and you should expect problems.
detect-buildid-unknown = The Steam build could not be read. This is normal for a folder you picked by hand.
detect-size = { $files } files, { $size }

## What the files turned out to be

detect-check-title = Checking the game files
detect-check-clean = Every file matches retail 1.6a.
detect-check-notices = Some files are not what retail 1.6a ships. This does not stop the install: the copy is yours and ReCoF replaces these files anyway. It is listed here so you know what you are copying.
verdict-retail = matches retail
verdict-patched = retail, with the 4 GB patch applied
verdict-unknown = not a file this launcher knows, SHA-256 { $sha256 }
verdict-missing = missing
detect-unexpected-title = Extra programs found in the game folder
detect-unexpected-help = Retail Cry of Fear does not ship these. They usually come from a community patch.
detect-mods-title = Known modifications

variant-cof-enhanced = the version the Cry of Fear Enhanced patch installs
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = Xash3D engine replacement
mod-xash-menu = Xash3D menu
mod-bunnymod-xt = Bunnymod XT

## Choosing where to install

install-title = Install ReCoF
install-destination = Install to
install-destination-help = A new folder, outside Steam. ReCoF needs a full copy of the game.
install-size-line = The copy needs { $size }. { $volume } has { $available } free.
install-space-ok = There is room.
install-space-short = There is not enough room on { $volume }. You need { $missing } more.
install-space-explain = Besides the copy, the launcher keeps room for the last few versions and for save backups, so you can roll back and so an update never has to start without a way back.

dest-inside-steam = That folder is inside the Steam folder. ReCoF has to live outside it, so a Steam update can never overwrite it.
dest-inside-steamapps = That folder is inside a Steam library's steamapps folder, where Steam manages the files. Choose somewhere else.
dest-protected = That folder needs administrator rights and is not a good place for a game. Choose somewhere in your own files.
dest-is-vanilla = That is the Steam copy of the game. ReCoF goes in a separate folder.
dest-drive-root = That is the root of a drive. Choose a folder inside it.
dest-not-empty = That folder already holds { $entries } other files. Choose an empty folder or a new one.
dest-unusable = That path cannot be used.

## Installing

install-copying = Copying the game: { $current }
install-copying-counted = Copying the game, { $done } of { $total }
install-applying = Applying ReCoF { $version }
install-finishing = Finishing
install-done = ReCoF { $version } is installed.
install-cancel-confirm = Stop the install? Everything copied so far will be removed. Your Steam copy is not affected.
install-cancelled = The install was cancelled and nothing was left behind.
install-offer-import = Bring your saves, settings and unlocks over from the Steam copy
install-offer-shortcuts = Create a desktop shortcut
install-release-title = What gets installed
install-release-version = ReCoF { $version }
install-release-download = It is downloaded from GitHub and checked against its signature before anything is copied.
install-release-stored = Already downloaded and checked.
install-release-file = From { $name }
install-release-file-help = The signed manifest next to the zip is checked before anything is copied.
install-release-file-clear = Use a published release instead
install-release-checking = Looking for the latest release
install-release-offline = Could not reach GitHub. Try again, or install from a file.
install-release-try-beta = Nothing has been published on the stable channel yet. Testers can switch to beta.

## The main screen

home-play = Play ReCoF
home-play-vanilla = Play vanilla
home-no-install = ReCoF is not installed yet
home-install-now = Install ReCoF
home-checking = Checking for updates...
home-up-to-date = Up to date
home-update-available = ReCoF { $version } is available
home-launcher-update-available = A new version of the launcher is available: { $version }
home-offline = Could not reach GitHub. The launcher works without it.

## Starting the game

launch-steam-not-running = Steam is not running
launch-steam-not-running-help = Cry of Fear uses Steam while it runs. Start Steam first.
launch-open-steam = Open Steam
launch-start-anyway = Start anyway
launch-starting = Starting ReCoF...
launch-failed = ReCoF could not be started.
launch-already-running = ReCoF is already running.

launch-options-title = Launch options
launch-resolution = Resolution
launch-resolution-default = As the game decides
launch-windowed = Windowed
launch-widescreen = Widescreen
launch-3dmenu = 3D menu
launch-3dmenu-help = Retail starts with the 3D menu off. Turning it on uses the original animated menu.
launch-extra-args = Extra arguments
launch-extra-args-help = Passed to cof.exe as they are. Leave this empty unless you know what you need.
launch-preview = Command line

launch-cvars-title = ReCoF options
launch-cvars-none = This version of ReCoF ships no options of its own.
launch-cvars-from-release = These come from the version of ReCoF you have installed.
launch-cvars-defaults = Back to the defaults

## Updates

update-title = Update
update-available-title = ReCoF { $version } is available
update-installed-version = You have { $version }
update-changelog = What changed
update-no-changelog = This release has no notes.
update-install = Update now
update-later = Later
update-downloading = Downloading { $name }
update-verifying = Checking the signature and hashes
update-extracting = Unpacking
update-applying = Applying the update
update-done = Updated to ReCoF { $version }.
update-backup-first = Backing up your saves first
update-launcher-first = This version of ReCoF needs launcher { $needed } or newer. Update the launcher first.
update-launcher-title = Launcher update
update-launcher-restart = The launcher will restart to finish updating.
update-from-file = Install from a file
update-from-file-help = Choose a release zip. Its signed manifest must be next to it.
changelog-note = Note
changelog-tip = Tip
changelog-important = Important
changelog-warning = Warning
changelog-caution = Caution

channel-title = Release channel
channel-stable = Stable
channel-beta = Beta
channel-beta-help = Beta includes prereleases. Expect problems and keep your backups.

## Rolling back

rollback-title = Go back to an earlier version
rollback-none = No earlier version is kept.
rollback-to = Go back to { $version }
rollback-offer-backup = Restore the saves backed up before that update
rollback-done = ReCoF { $version } has been restored.

## Maintenance

maintain-title = Maintenance
maintain-verify = Verify and repair
maintain-verify-help = Hash every ReCoF file against the manifest and write back anything that does not match.
maintain-backup = Back up saves
maintain-restore = Restore saves
maintain-import = Import progress from the Steam copy
maintain-move = Move the installation
maintain-uninstall = Uninstall
maintain-shortcuts = Shortcuts
maintain-logs = Logs

verify-running = Checking { $done } of { $total }
verify-intact = Every file matches.
verify-broken = { $count } files do not match.
verify-repair = Repair them
verify-repaired = { $count } files were written again.
verify-cannot-repair = The version these files came from is not in the store any more. Reinstall or update to repair.

## Save backups

backup-title = Save backups
backup-what = Backed up: saves, settings, unlocks and your config files.
backup-create = Back up now
backup-created = Backed up to { $name }
backup-none = There are no backups yet.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = Restore { $name }? The saves in your ReCoF copy will be replaced.
backup-restored = The backup was restored.
backup-verify-failed = The backup could not be read back after it was written, so it is not trustworthy. Nothing was changed.
backup-auto = Taken automatically before updating to { $version }

## Importing from the Steam copy

import-title = Import progress
import-explain = Copies your saves, settings and unlocks from the Steam installation into the ReCoF copy. Nothing is written into the Steam installation, and nothing is ever copied back the other way.
import-nothing-found = The Steam copy has no saves to import.
import-overwrite-title = This will replace files you already have
import-overwrite-body = { $count } files in the ReCoF copy will be replaced. A backup is taken first.
import-done = { $count } files were imported.

## Moving

move-title = Move the installation
move-to = Move to
move-running = Moving
move-done = The installation is now at { $path }.
move-same-volume = On the same drive this is instant.

## Uninstalling

uninstall-title = Uninstall ReCoF
uninstall-explain = This removes the ReCoF copy and the stored versions. Your Steam installation is not touched.
uninstall-export-first = Export my saves first
uninstall-keep-backups = Keep my save backups
uninstall-confirm = Remove { $path }?
uninstall-done = ReCoF was removed.

## Shortcuts

shortcut-desktop = Desktop shortcut
shortcut-start-menu = Start menu shortcut
shortcut-steam = Add ReCoF to Steam as a non Steam game
shortcut-steam-help = This is the only thing the launcher writes outside its own folders. Steam has to be closed, and a copy of shortcuts.vdf is kept first.
shortcut-steam-needs-steam-closed = Close Steam first. Steam rewrites this file when it exits, which would undo the change.
shortcut-steam-added = ReCoF was added to Steam. Restart Steam to see it.
shortcut-created = Shortcut created.

## Reporting a bug

bug-title = Report a bug
bug-explain = This collects the versions, the result of the last verification, your system details and the last lines of the log. Look through it, remove anything you would rather not share, and the launcher opens a prefilled issue in your browser.
bug-never-sends = Nothing is sent anywhere. The launcher only opens your browser.
bug-collecting = Collecting
bug-open-issue = Open the issue in my browser
bug-copied = Copied to the clipboard.

## Settings

settings-title = Settings
settings-language = Language
settings-check-updates = Check for updates when the launcher starts
settings-discord = Discord Rich Presence
settings-discord-help = Shows that you are playing ReCoF on your Discord profile, while the game is open. Off by default.
settings-keep-versions = Versions to keep for rolling back
settings-open-logs = Open the log folder
settings-about = About

## Recovery after an interrupted operation

recovery-rolled-back = An operation that was interrupted has been undone. The installation is as it was before it started.
recovery-rolled-forward = An operation that was interrupted has been finished. The installation is at { $version }.

## Errors

err-io = A file could not be read or written: { $path }
err-not-supported = This is not available on this system.
err-registry = The Windows registry key { $key } could not be read.
err-platform-api = { $api } failed.
err-no-game-exe = { $path } does not hold cof.exe, so it is not a Cry of Fear folder.
err-no-game-dir = { $path } does not hold a cryoffear folder, so it is not a Cry of Fear folder.
err-journal = An install operation could not be completed. Nothing is half done: the launcher will finish or undo it on the next start.
err-interrupted = The operation was stopped.

err-no-trusted-keys = This build of the launcher has no signing key, so it cannot verify a release. Nothing was installed.
err-bad-signature = The signature on this release does not check out. Nothing was installed. Download it again, and if it keeps failing, report it.
err-bad-manifest = This release describes itself in a way the launcher will not accept. Nothing was installed.
err-launcher-too-old = This release needs a newer launcher. Update the launcher first.
err-wrong-build = This release is built for a different version of Cry of Fear.
err-hash-mismatch = A downloaded file is not what it should be. Nothing was installed.

err-network = GitHub could not be reached. Check your connection and try again.
err-github = GitHub answered with an error: { $detail }
err-rate-limited = GitHub is rate limiting this connection. Try again in a few minutes.
err-no-release = No release has been published on this channel yet.
err-command-line-too-long = The command line would be { $length } characters long, and the game takes at most { $limit }. Shorten the extra arguments.

## Settings, in tabs

settings-tab-look = Appearance
settings-tab-language = Language
settings-tab-updates = Updates
settings-tab-behaviour = Behaviour
settings-tab-about = About

## Appearance

look-title = Appearance
look-show-hero = Use the background picture
look-hero-dim = How dark the background is
look-hero-dim-help = Far enough that the text over it stays readable.
look-grain = Film grain
look-command-line = Show the command line on the main screen

look-artwork-title = Your own artwork
look-artwork-help = The launcher ships no pictures of the game. Put your own in this folder and it uses them: hero.png for the background, logo.png for the title, icon.png for the window icon. They stay on this machine and never go into a release.
look-open-theme = Open the artwork folder
look-reload-art = Reload artwork
look-using-art = Using your artwork: { $pieces }
look-no-art = No artwork found, so the launcher is drawing its own.

## Behaviour

behaviour-minimize = Minimise the launcher when the game starts
behaviour-close = Close the launcher when the game starts
behaviour-close-help = Takes precedence over minimising.

## About

about-what = ReCoF Launcher installs, updates and starts ReCoF, the improved recompilation of Cry of Fear. It copies your Steam installation to a folder of your choosing and never writes inside the Steam one.
about-made-by = Made by { $author }, who also makes ReCoF itself.
about-version = Launcher { $version }
about-licence = MIT licensed. The source is at { $url }.
about-recof = ReCoF is at { $url }.
about-font = The title is set in Archivo Black, under the SIL Open Font License 1.1.
about-not-affiliated = Not affiliated with Team Psykskallar. Cry of Fear belongs to its authors; this launcher only installs a recompilation of it from your own copy.

## Artwork credits

look-credits-title = Artwork credits
look-credits-none = No credits came with this artwork.
look-credits-help = The artwork is not part of the launcher. It comes from the ReCoF release and belongs to whoever made it.
look-forget-art = Remove the artwork
# The main screen: the rail, the stat rows and the menu beside Play.
nav-home = Home
nav-options = Launch options
nav-maintain = Maintenance
nav-bug = Report a problem
nav-settings = Settings
nav-folder = Open the install folder
home-more = More
home-stat-last-played = Last played
home-stat-playtime = Total playtime
home-stat-version = Installed version
home-played-never = Never
home-played-today = Today
home-played-yesterday = Yesterday
home-played-days = { $days } days ago
home-playtime-none = Nothing yet
home-playtime-minutes = { $minutes } min
home-playtime-hours = { $hours } h { $minutes } min
settings-discord-unavailable = This build has no Discord application id, so the presence cannot be shown.
updates-check-title = Checking for updates
updates-check-now = Check now
