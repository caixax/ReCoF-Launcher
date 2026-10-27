# ReCoF Launcher, Deutsch.
#
# ERSTENTWURF: Diese Übersetzung wurde noch nicht von einer
# Muttersprachlerin oder einem Muttersprachler geprüft. Korrekturen sind
# sehr willkommen: https://github.com/caixax/ReCoF-Launcher
#
# Stil: schlichte Sätze, keine Geviertstriche und keine Halbgeviertstriche.

app-name = ReCoF Launcher
app-tagline = Installer und Starter für ReCoF

## Schaltflächen und Wörter, die überall vorkommen

button-next = Weiter
button-back = Zurück
button-cancel = Abbrechen
button-close = Schließen
button-continue = Fortfahren
button-ok = OK
button-yes = Ja
button-no = Nein
button-browse = Durchsuchen...
button-retry = Erneut versuchen
button-copy = Kopieren
button-open-folder = Ordner öffnen
button-finish = Fertig
button-skip = Überspringen
button-details = Details
word-yes = ja
word-no = nein
word-unknown = unbekannt

## Erster Start: Sprache

firstrun-language-title = Sprache auswählen
firstrun-language-help = Das lässt sich später in den Einstellungen ändern.
language-not-reviewed = Erstentwurf, noch nicht von Muttersprachlern geprüft
language-not-reviewed-notice = Diese Übersetzung ist ein Erstentwurf und wurde noch nicht von Muttersprachlern geprüft. Korrekturen sind willkommen.

## Erster Start: worum es geht

intro-1-title = Was ReCoF ist
intro-1-body = ReCoF ist eine verbesserte Neukompilierung von Cry of Fear. Dieser Launcher installiert es, hält es aktuell und startet es. ReCoF und dieser Launcher sind eigenständige Projekte, getrennt vom Spiel selbst.
intro-2-title = Deine Steam-Kopie bleibt unberührt
intro-2-body = Der Launcher liest deine Steam-Installation und kopiert sie in einen Ordner deiner Wahl. Er schreibt, löscht und benennt nichts im Steam-Ordner um, Cry of Fear funktioniert also genau wie bisher und du kannst jederzeit dorthin zurück.
intro-3-title = Eine eigene Kopie, und dein Fortschritt
intro-3-body = Weil ReCoF in einer eigenen Kopie liegt, braucht es Platz für das ganze Spiel, etwa { $size }. Deine Spielstände, Einstellungen und Freischaltungen kannst du bei der Installation oder später über das Menü aus der Steam-Kopie übernehmen. In die andere Richtung wird nie etwas kopiert.

## Das Spiel finden

detect-title = Cry of Fear suchen
detect-searching = Steam-Installation wird gesucht...
detect-found = Gefunden unter { $path }
detect-not-found = Cry of Fear wurde nicht gefunden
detect-not-found-help = Vielleicht ist Steam nicht installiert, oder das Spiel liegt in einem Ordner, den der Launcher nicht sieht. Wähle den Ordner, in dem cof.exe liegt.
detect-pick-folder = Spielordner auswählen
detect-pick-folder-help = Der Ordner, in dem cof.exe liegt
detect-library = Steam-Bibliothek: { $path }
detect-buildid-ok = Build { $buildid }, genau die, für die ReCoF gebaut ist
detect-buildid-wrong = Das ist Build { $found }. ReCoF ist für Build { $expected } gebaut. Es kann sein, dass es nicht richtig läuft, und Probleme sind zu erwarten.
detect-buildid-unknown = Der Steam-Build konnte nicht gelesen werden. Bei einem von Hand gewählten Ordner ist das normal.
detect-size = { $files } Dateien, { $size }

## Was die Dateien sind

detect-check-title = Spieldateien werden geprüft
detect-check-clean = Alle Dateien entsprechen der Verkaufsversion 1.6a.
detect-check-notices = Einige Dateien sind nicht die der Verkaufsversion 1.6a. Das hält die Installation nicht auf: die Kopie gehört dir und ReCoF ersetzt diese Dateien ohnehin. Es steht hier, damit du weißt, was du kopierst.
verdict-retail = entspricht der Verkaufsversion
verdict-patched = Verkaufsversion mit dem 4-GB-Patch
verdict-unknown = keine dem Launcher bekannte Datei, SHA-256 { $sha256 }
verdict-missing = fehlt
detect-unexpected-title = Zusätzliche Programme im Spielordner
detect-unexpected-help = Die Verkaufsversion von Cry of Fear liefert diese nicht mit. Sie stammen meist aus einem Community-Patch.
detect-mods-title = Bekannte Modifikationen

variant-cof-enhanced = die Fassung, die der Patch Cry of Fear Enhanced installiert
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = Austausch der Engine durch Xash3D
mod-xash-menu = Xash3D-Menü
mod-bunnymod-xt = Bunnymod XT

## Wohin installiert wird

install-title = ReCoF installieren
install-destination = Installieren nach
install-destination-help = Ein neuer Ordner außerhalb von Steam. ReCoF braucht eine vollständige Kopie des Spiels.
install-size-line = Die Kopie braucht { $size }. Auf { $volume } sind { $available } frei.
install-space-ok = Es ist genug Platz da.
install-space-short = Auf { $volume } ist nicht genug Platz. Es fehlen { $missing }.
install-space-explain = Außer der Kopie hält der Launcher Platz für die letzten Versionen und für Sicherungen der Spielstände frei, damit du zurückgehen kannst und ein Update nie ohne Rückweg beginnt.

dest-inside-steam = Dieser Ordner liegt im Steam-Ordner. ReCoF muss außerhalb liegen, damit ein Steam-Update es nie überschreiben kann.
dest-inside-steamapps = Dieser Ordner liegt im steamapps-Ordner einer Steam-Bibliothek, wo Steam die Dateien verwaltet. Wähle einen anderen Ort.
dest-protected = Dieser Ordner braucht Administratorrechte und ist kein guter Ort für ein Spiel. Wähle etwas in deinen eigenen Dateien.
dest-is-vanilla = Das ist die Steam-Kopie des Spiels. ReCoF kommt in einen eigenen Ordner.
dest-drive-root = Das ist die Wurzel eines Laufwerks. Wähle einen Ordner darin.
dest-not-empty = In diesem Ordner liegen bereits { $entries } andere Dateien. Wähle einen leeren oder einen neuen Ordner.
dest-unusable = Dieser Pfad lässt sich nicht verwenden.

## Installation

install-copying = Spiel wird kopiert: { $current }
install-copying-counted = Spiel wird kopiert, { $done } von { $total }
install-applying = ReCoF { $version } wird angewendet
install-finishing = Wird abgeschlossen
install-done = ReCoF { $version } ist installiert.
install-cancel-confirm = Installation abbrechen? Alles bisher Kopierte wird entfernt. Deine Steam-Kopie bleibt unberührt.
install-cancelled = Die Installation wurde abgebrochen und hat nichts hinterlassen.
install-offer-import = Spielstände, Einstellungen und Freischaltungen aus der Steam-Kopie übernehmen
install-offer-shortcuts = Verknüpfung auf dem Desktop anlegen
install-release-title = Was installiert wird
install-release-version = ReCoF { $version }
install-release-download = Wird von GitHub heruntergeladen und vor dem Kopieren anhand der Signatur geprüft.
install-release-stored = Bereits heruntergeladen und geprüft.
install-release-file = Aus { $name }
install-release-file-help = Das signierte Manifest neben der ZIP-Datei wird vor dem Kopieren geprüft.
install-release-file-clear = Stattdessen eine veröffentlichte Version verwenden
install-release-checking = Suche nach der neuesten Version
install-release-offline = GitHub ist nicht erreichbar. Versuche es erneut oder installiere aus einer Datei.
install-release-try-beta = Im stabilen Kanal ist noch nichts veröffentlicht. Tester können zu Beta wechseln.

## Der Hauptbildschirm

home-play = ReCoF spielen
home-play-vanilla = Original spielen
home-no-install = ReCoF ist noch nicht installiert
home-install-now = ReCoF installieren
home-checking = Nach Updates wird gesucht...
home-up-to-date = Alles aktuell
home-update-available = ReCoF { $version } ist verfügbar
home-launcher-update-available = Es gibt eine neue Version des Launchers: { $version }
home-offline = GitHub war nicht erreichbar. Der Launcher funktioniert auch ohne.

## Das Spiel starten

launch-steam-not-running = Steam läuft nicht
launch-steam-not-running-help = Cry of Fear nutzt Steam während es läuft. Starte zuerst Steam.
launch-open-steam = Steam öffnen
launch-start-anyway = Trotzdem starten
launch-starting = ReCoF wird gestartet...
launch-failed = ReCoF konnte nicht gestartet werden.
launch-already-running = ReCoF läuft bereits.

launch-options-title = Startoptionen
launch-resolution = Auflösung
launch-resolution-default = Wie das Spiel entscheidet
launch-windowed = Im Fenster
launch-widescreen = Breitbild
launch-3dmenu = 3D-Menü
launch-3dmenu-help = Die Verkaufsversion startet mit abgeschaltetem 3D-Menü. Eingeschaltet wird das ursprüngliche animierte Menü verwendet.
launch-extra-args = Zusätzliche Argumente
launch-extra-args-help = Werden unverändert an cof.exe weitergegeben. Lass das leer, wenn du nicht weißt, was du brauchst.
launch-preview = Befehlszeile

launch-cvars-title = ReCoF-Optionen
launch-cvars-none = Diese Version von ReCoF bringt keine eigenen Optionen mit.
launch-cvars-from-release = Sie stammen aus der installierten Version von ReCoF.
launch-cvars-defaults = Auf Standardwerte zurücksetzen

## Updates

update-title = Update
update-available-title = ReCoF { $version } ist verfügbar
update-installed-version = Du hast { $version }
update-changelog = Was sich geändert hat
update-no-changelog = Zu dieser Version gibt es keine Anmerkungen.
update-install = Jetzt aktualisieren
update-later = Später
update-downloading = { $name } wird heruntergeladen
update-verifying = Signatur und Prüfsummen werden geprüft
update-extracting = Wird entpackt
update-applying = Update wird angewendet
update-done = Auf ReCoF { $version } aktualisiert.
update-backup-first = Zuerst werden deine Spielstände gesichert
update-launcher-first = Diese Version von ReCoF braucht den Launcher { $needed } oder neuer. Aktualisiere zuerst den Launcher.
update-launcher-title = Launcher-Update
update-launcher-restart = Der Launcher startet neu, um das Update abzuschließen.
update-from-file = Aus einer Datei installieren
update-from-file-help = Wähle ein Release-Zip. Das signierte Manifest muss daneben liegen.
changelog-note = Hinweis
changelog-tip = Tipp
changelog-important = Wichtig
changelog-warning = Warnung
changelog-caution = Vorsicht

channel-title = Veröffentlichungskanal
channel-stable = Stabil
channel-beta = Beta
channel-beta-help = Beta enthält Vorabversionen. Rechne mit Problemen und behalte deine Sicherungen.

## Zurückgehen

rollback-title = Zu einer älteren Version zurück
rollback-none = Es ist keine ältere Version aufbewahrt.
rollback-to = Zurück zu { $version }
rollback-offer-backup = Die vor jenem Update gesicherten Spielstände wiederherstellen
rollback-done = ReCoF { $version } wurde wiederhergestellt.

## Wartung

maintain-title = Wartung
maintain-verify = Prüfen und reparieren
maintain-verify-help = Jede ReCoF-Datei gegen das Manifest prüfen und alles neu schreiben, was nicht passt.
maintain-backup = Spielstände sichern
maintain-restore = Spielstände wiederherstellen
maintain-import = Fortschritt aus der Steam-Kopie übernehmen
maintain-move = Installation verschieben
maintain-uninstall = Deinstallieren
maintain-shortcuts = Verknüpfungen
maintain-logs = Protokolle

verify-running = { $done } von { $total } wird geprüft
verify-intact = Alle Dateien passen.
verify-broken = { $count } Dateien passen nicht.
verify-repair = Reparieren
verify-repaired = { $count } Dateien wurden neu geschrieben.
verify-cannot-repair = Die Version, aus der diese Dateien stammen, liegt nicht mehr im Speicher. Installiere neu oder aktualisiere, um zu reparieren.

## Sicherungen der Spielstände

backup-title = Sicherungen der Spielstände
backup-what = Gesichert werden: Spielstände, Einstellungen, Freischaltungen und deine Konfigurationsdateien.
backup-create = Jetzt sichern
backup-created = Gesichert als { $name }
backup-none = Es gibt noch keine Sicherungen.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = { $name } wiederherstellen? Die Spielstände in deiner ReCoF-Kopie werden ersetzt.
backup-restored = Die Sicherung wurde wiederhergestellt.
backup-verify-failed = Die Sicherung ließ sich nach dem Schreiben nicht wieder lesen, ihr ist also nicht zu trauen. Es wurde nichts geändert.
backup-auto = Automatisch vor dem Update auf { $version } erstellt

## Übernahme aus der Steam-Kopie

import-title = Fortschritt übernehmen
import-explain = Kopiert deine Spielstände, Einstellungen und Freischaltungen aus der Steam-Installation in die ReCoF-Kopie. In die Steam-Installation wird nichts geschrieben, und in die andere Richtung wird nie etwas kopiert.
import-nothing-found = In der Steam-Kopie gibt es keine Spielstände zum Übernehmen.
import-overwrite-title = Dabei werden vorhandene Dateien ersetzt
import-overwrite-body = { $count } Dateien in der ReCoF-Kopie werden ersetzt. Vorher wird gesichert.
import-done = { $count } Dateien wurden übernommen.

## Verschieben

move-title = Installation verschieben
move-to = Verschieben nach
move-running = Wird verschoben
move-done = Die Installation liegt jetzt unter { $path }.
move-same-volume = Auf demselben Laufwerk geht das sofort.

## Deinstallieren

uninstall-title = ReCoF deinstallieren
uninstall-explain = Das entfernt die ReCoF-Kopie und die gespeicherten Versionen. Deine Steam-Installation bleibt unberührt.
uninstall-export-first = Vorher meine Spielstände exportieren
uninstall-keep-backups = Meine Sicherungen behalten
uninstall-confirm = { $path } entfernen?
uninstall-done = ReCoF wurde entfernt.

## Verknüpfungen

shortcut-desktop = Verknüpfung auf dem Desktop
shortcut-start-menu = Verknüpfung im Startmenü
shortcut-steam = ReCoF als Nicht-Steam-Spiel zu Steam hinzufügen
shortcut-steam-help = Das ist das Einzige, was der Launcher außerhalb seiner eigenen Ordner schreibt. Steam muss geschlossen sein, und vorher wird eine Kopie von shortcuts.vdf angelegt.
shortcut-steam-needs-steam-closed = Schließe zuerst Steam. Steam schreibt diese Datei beim Beenden neu und würde die Änderung rückgängig machen.
shortcut-steam-added = ReCoF wurde zu Steam hinzugefügt. Starte Steam neu, um es zu sehen.
shortcut-created = Verknüpfung angelegt.

## Einen Fehler melden

bug-title = Fehler melden
bug-explain = Hier werden die Versionen, das Ergebnis der letzten Prüfung, Angaben zu deinem System und die letzten Zeilen des Protokolls gesammelt. Sieh es durch, entferne, was du lieber nicht teilen möchtest, und der Launcher öffnet einen vorausgefüllten Bericht im Browser.
bug-never-sends = Es wird nichts irgendwohin gesendet. Der Launcher öffnet nur deinen Browser.
bug-collecting = Wird gesammelt
bug-open-issue = Den Bericht im Browser öffnen
bug-copied = In die Zwischenablage kopiert.

## Einstellungen

settings-title = Einstellungen
settings-language = Sprache
settings-check-updates = Beim Start des Launchers nach Updates suchen
settings-discord = Discord Rich Presence
settings-discord-help = Zeigt in deinem Discord-Profil an, dass du ReCoF spielst, solange das Spiel läuft. Standardmäßig aus.
settings-keep-versions = Versionen, die zum Zurückgehen aufbewahrt werden
settings-open-logs = Protokollordner öffnen
settings-about = Über

## Wiederherstellung nach einem Abbruch

recovery-rolled-back = Ein abgebrochener Vorgang wurde rückgängig gemacht. Die Installation ist so wie vor dem Beginn.
recovery-rolled-forward = Ein abgebrochener Vorgang wurde abgeschlossen. Die Installation ist auf Version { $version }.

## Fehler

err-io = Eine Datei konnte nicht gelesen oder geschrieben werden: { $path }
err-not-supported = Das ist auf diesem System nicht verfügbar.
err-registry = Der Windows-Registrierungsschlüssel { $key } konnte nicht gelesen werden.
err-platform-api = { $api } ist fehlgeschlagen.
err-no-game-exe = In { $path } liegt keine cof.exe, es ist also kein Cry-of-Fear-Ordner.
err-no-game-dir = In { $path } liegt kein cryoffear-Ordner, es ist also kein Cry-of-Fear-Ordner.
err-journal = Ein Installationsvorgang konnte nicht abgeschlossen werden. Nichts bleibt halb fertig: der Launcher schließt ihn beim nächsten Start ab oder macht ihn rückgängig.
err-interrupted = Der Vorgang wurde angehalten.

err-no-trusted-keys = Diese Fassung des Launchers hat keinen Signaturschlüssel und kann daher keine Veröffentlichung prüfen. Es wurde nichts installiert.
err-bad-signature = Die Signatur dieser Veröffentlichung stimmt nicht. Es wurde nichts installiert. Lade sie erneut herunter, und melde es, wenn es weiter fehlschlägt.
err-bad-manifest = Diese Veröffentlichung beschreibt sich auf eine Weise, die der Launcher nicht annimmt. Es wurde nichts installiert.
err-launcher-too-old = Diese Veröffentlichung braucht einen neueren Launcher. Aktualisiere zuerst den Launcher.
err-wrong-build = Diese Veröffentlichung ist für eine andere Version von Cry of Fear gebaut.
err-hash-mismatch = Eine heruntergeladene Datei ist nicht das, was sie sein sollte. Es wurde nichts installiert.

err-network = GitHub war nicht erreichbar. Prüfe deine Verbindung und versuche es erneut.
err-github = GitHub hat mit einem Fehler geantwortet: { $detail }
err-rate-limited = GitHub drosselt diese Verbindung. Versuche es in ein paar Minuten erneut.
err-no-release = In diesem Kanal wurde noch nichts veröffentlicht.
err-command-line-too-long = Die Befehlszeile wäre { $length } Zeichen lang, das Spiel nimmt höchstens { $limit }. Kürze die zusätzlichen Argumente.

## Einstellungen, in Reitern

settings-tab-look = Aussehen
settings-tab-language = Sprache
settings-tab-updates = Updates
settings-tab-behaviour = Verhalten
settings-tab-about = Über

## Aussehen

look-title = Aussehen
look-show-hero = Hintergrundbild verwenden
look-hero-dim = Wie stark der Hintergrund abgedunkelt wird
look-hero-dim-help = So weit, dass der Text darüber lesbar bleibt.
look-grain = Filmkorn
look-command-line = Die Befehlszeile auf dem Hauptbildschirm zeigen

look-artwork-title = Deine eigenen Bilder
look-artwork-help = Der Launcher bringt keine Bilder des Spiels mit. Lege deine in diesen Ordner und er verwendet sie: hero.png als Hintergrund, logo.png als Titel, icon.png als Fenstersymbol. Sie bleiben auf diesem Rechner und kommen nie in eine Veröffentlichung.
look-open-theme = Bilderordner öffnen
look-reload-art = Bilder neu laden
look-using-art = Deine Bilder werden verwendet: { $pieces }
look-no-art = Keine Bilder gefunden, also zeichnet der Launcher seine eigenen.

## Verhalten

behaviour-minimize = Den Launcher minimieren, wenn das Spiel startet
behaviour-close = Den Launcher schließen, wenn das Spiel startet
behaviour-close-help = Hat Vorrang vor dem Minimieren.

## Über

about-what = ReCoF Launcher installiert, aktualisiert und startet ReCoF, die verbesserte Neukompilierung von Cry of Fear. Er kopiert deine Steam-Installation in einen Ordner deiner Wahl und schreibt nie in den von Steam.
about-made-by = Gemacht von { $author }, der auch ReCoF selbst macht.
about-version = Launcher { $version }
about-licence = MIT-Lizenz. Der Quelltext liegt unter { $url }.
about-recof = ReCoF liegt unter { $url }.
about-font = Der Titel ist in Archivo Black gesetzt, unter der SIL Open Font License 1.1.
about-not-affiliated = Keine Verbindung zu Team Psykskallar. Cry of Fear gehört seinen Urhebern; dieser Launcher installiert nur eine Neukompilierung aus deiner eigenen Kopie.

## Bildnachweis

look-credits-title = Bildnachweis
look-credits-none = Zu diesen Bildern kam kein Nachweis mit.
look-credits-help = Die Bilder gehören nicht zum Launcher. Sie kommen aus der ReCoF-Veröffentlichung und gehören denen, die sie gemacht haben.
look-forget-art = Bilder entfernen
nav-home = Start
nav-options = Startoptionen
nav-maintain = Wartung
nav-bug = Problem melden
nav-settings = Einstellungen
nav-folder = Installationsordner öffnen
home-more = Mehr
home-stat-last-played = Zuletzt gespielt
home-stat-playtime = Gesamte Spielzeit
home-stat-version = Installierte Version
home-played-never = Nie
home-played-today = Heute
home-played-yesterday = Gestern
home-played-days = Vor { $days } Tagen
home-playtime-none = Noch nichts
home-playtime-minutes = { $minutes } Min
home-playtime-hours = { $hours } Std { $minutes } Min
settings-discord-unavailable = Dieser Build hat keine Discord Anwendungs ID, daher kann die Präsenz nicht angezeigt werden.
