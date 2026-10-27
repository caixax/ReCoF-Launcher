# ReCoF Launcher, polski.
#
# PIERWSZA WERSJA: to tłumaczenie nie zostało jeszcze sprawdzone przez
# osobę mówiącą po polsku od urodzenia. Poprawki są mile widziane:
# https://github.com/caixax/ReCoF-Launcher
#
# Styl: proste zdania, bez pauz ani półpauz.

app-name = ReCoF Launcher
app-tagline = Instalator i program uruchamiający ReCoF

## Przyciski i słowa, które pojawiają się wszędzie

button-next = Dalej
button-back = Wstecz
button-cancel = Anuluj
button-close = Zamknij
button-continue = Kontynuuj
button-ok = OK
button-yes = Tak
button-no = Nie
button-browse = Przeglądaj...
button-retry = Spróbuj ponownie
button-copy = Kopiuj
button-open-folder = Otwórz folder
button-finish = Zakończ
button-skip = Pomiń
button-details = Szczegóły
word-yes = tak
word-no = nie
word-unknown = nieznane

## Pierwsze uruchomienie: język

firstrun-language-title = Wybierz język
firstrun-language-help = Można to zmienić później w ustawieniach.
language-not-reviewed = Pierwsza wersja, jeszcze niesprawdzona przez native speakera
language-not-reviewed-notice = To tłumaczenie jest pierwszą wersją i nie zostało jeszcze sprawdzone przez native speakera. Poprawki są mile widziane.

## Pierwsze uruchomienie: o co chodzi

intro-1-title = Czym jest ReCoF
intro-1-body = ReCoF to ulepszona rekompilacja Cry of Fear. Ten program ją instaluje, aktualizuje i uruchamia. ReCoF i ten program to projekty oddzielne od samej gry.
intro-2-title = Twoja kopia ze Steama pozostaje nietknięta
intro-2-body = Program czyta instalację ze Steama i kopiuje ją do folderu, który wskażesz. Nigdy niczego nie zapisuje, nie usuwa ani nie zmienia nazw w folderze Steama, więc Cry of Fear działa dokładnie tak jak dotąd i możesz do niego wrócić w każdej chwili.
intro-3-title = Osobna kopia i twoje postępy
intro-3-body = Ponieważ ReCoF żyje we własnej kopii, potrzebuje miejsca na całą grę, około { $size }. Zapisy, ustawienia i odblokowane rzeczy można przenieść z kopii ze Steama podczas instalacji albo później z menu. W drugą stronę nic nigdy nie jest kopiowane.

## Znajdowanie gry

detect-title = Szukanie Cry of Fear
detect-searching = Szukanie instalacji Steama...
detect-found = Znaleziono w { $path }
detect-not-found = Nie znaleziono Cry of Fear
detect-not-found-help = Steam może nie być zainstalowany albo gra leży w folderze, którego program nie widzi. Wskaż folder, w którym jest cof.exe.
detect-pick-folder = Wybierz folder gry
detect-pick-folder-help = Folder, w którym jest cof.exe
detect-library = Biblioteka Steama: { $path }
detect-buildid-ok = Build { $buildid }, ten, pod który zbudowano ReCoF
detect-buildid-wrong = To jest build { $found }. ReCoF zbudowano pod build { $expected }. Może nie działać poprawnie i należy spodziewać się problemów.
detect-buildid-unknown = Nie udało się odczytać buildu ze Steama. Przy folderze wskazanym ręcznie to normalne.
detect-size = { $files } plików, { $size }

## Czym okazały się pliki

detect-check-title = Sprawdzanie plików gry
detect-check-clean = Wszystkie pliki zgadzają się z wersją handlową 1.6a.
detect-check-notices = Niektóre pliki nie są tymi z wersji handlowej 1.6a. Nie blokuje to instalacji: kopia należy do ciebie, a ReCoF i tak zastępuje te pliki. Jest to wypisane, żebyś wiedział, co kopiujesz.
verdict-retail = zgodny z wersją handlową
verdict-patched = wersja handlowa z łatką 4 GB
verdict-unknown = plik nieznany programowi, SHA-256 { $sha256 }
verdict-missing = brakuje
detect-unexpected-title = Dodatkowe programy w folderze gry
detect-unexpected-help = Wersja handlowa Cry of Fear ich nie zawiera. Zwykle pochodzą z łatki społecznościowej.
detect-mods-title = Znane modyfikacje

variant-cof-enhanced = wersja instalowana przez łatkę Cry of Fear Enhanced
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = podmiana silnika na Xash3D
mod-xash-menu = menu Xash3D
mod-bunnymod-xt = Bunnymod XT

## Gdzie zainstalować

install-title = Zainstaluj ReCoF
install-destination = Zainstaluj w
install-destination-help = Nowy folder, poza Steamem. ReCoF potrzebuje pełnej kopii gry.
install-size-line = Kopia wymaga { $size }. Na { $volume } zostało { $available }.
install-space-ok = Miejsca wystarczy.
install-space-short = Na { $volume } brakuje miejsca. Brakuje { $missing }.
install-space-explain = Oprócz kopii program rezerwuje miejsce na kilka ostatnich wersji i na kopie zapasowe zapisów, żebyś mógł się cofnąć i żeby aktualizacja nigdy nie zaczynała się bez drogi powrotnej.

dest-inside-steam = Ten folder jest wewnątrz folderu Steama. ReCoF musi być poza nim, żeby aktualizacja Steama nigdy go nie nadpisała.
dest-inside-steamapps = Ten folder jest w steamapps biblioteki Steama, gdzie plikami zarządza Steam. Wybierz inne miejsce.
dest-protected = Ten folder wymaga uprawnień administratora i nie jest dobrym miejscem na grę. Wybierz coś w swoich własnych plikach.
dest-is-vanilla = To jest kopia gry ze Steama. ReCoF trafia do osobnego folderu.
dest-drive-root = To jest katalog główny dysku. Wybierz folder wewnątrz niego.
dest-not-empty = W tym folderze jest już { $entries } innych plików. Wybierz pusty albo nowy folder.
dest-unusable = Tej ścieżki nie da się użyć.

## Instalowanie

install-copying = Kopiowanie gry: { $current }
install-copying-counted = Kopiowanie gry, { $done } z { $total }
install-applying = Stosowanie ReCoF { $version }
install-finishing = Kończenie
install-done = ReCoF { $version } jest zainstalowany.
install-cancel-confirm = Przerwać instalację? Wszystko, co dotąd skopiowano, zostanie usunięte. Twoja kopia ze Steama nie jest ruszana.
install-cancelled = Instalacja została przerwana i nic po niej nie zostało.
install-offer-import = Przenieś moje zapisy, ustawienia i odblokowane rzeczy z kopii ze Steama
install-offer-shortcuts = Utwórz skrót na pulpicie
install-release-title = Co zostanie zainstalowane
install-release-version = ReCoF { $version }
install-release-download = Zostanie pobrane z GitHuba i sprawdzone podpisem, zanim cokolwiek zostanie skopiowane.
install-release-stored = Już pobrane i sprawdzone.
install-release-file = Z pliku { $name }
install-release-file-help = Podpisany manifest obok pliku zip jest sprawdzany, zanim cokolwiek zostanie skopiowane.
install-release-file-clear = Użyj zamiast tego opublikowanej wersji
install-release-checking = Szukanie najnowszej wersji
install-release-offline = Nie można połączyć się z GitHubem. Spróbuj ponownie lub zainstaluj z pliku.
install-release-try-beta = Na kanale stabilnym nic jeszcze nie opublikowano. Testerzy mogą przełączyć się na betę.

## Ekran główny

home-play = Zagraj w ReCoF
home-play-vanilla = Zagraj w oryginał
home-no-install = ReCoF nie jest jeszcze zainstalowany
home-install-now = Zainstaluj ReCoF
home-checking = Sprawdzanie aktualizacji...
home-up-to-date = Wszystko aktualne
home-update-available = Dostępny jest ReCoF { $version }
home-launcher-update-available = Dostępna jest nowa wersja programu: { $version }
home-offline = Nie udało się połączyć z GitHubem. Program działa i bez tego.

## Uruchamianie gry

launch-steam-not-running = Steam nie jest uruchomiony
launch-steam-not-running-help = Cry of Fear korzysta ze Steama w trakcie działania. Najpierw uruchom Steama.
launch-open-steam = Otwórz Steama
launch-start-anyway = Uruchom mimo to
launch-starting = Uruchamianie ReCoF...
launch-failed = Nie udało się uruchomić ReCoF.
launch-already-running = ReCoF jest już uruchomiony.

launch-options-title = Opcje uruchamiania
launch-resolution = Rozdzielczość
launch-resolution-default = Taka, jaką wybierze gra
launch-windowed = W oknie
launch-widescreen = Panoramiczny
launch-3dmenu = Menu 3D
launch-3dmenu-help = Wersja handlowa startuje z wyłączonym menu 3D. Po włączeniu używane jest oryginalne animowane menu.
launch-extra-args = Dodatkowe argumenty
launch-extra-args-help = Przekazywane do cof.exe bez zmian. Zostaw puste, jeśli nie wiesz, czego potrzebujesz.
launch-preview = Wiersz poleceń

launch-cvars-title = Opcje ReCoF
launch-cvars-none = Ta wersja ReCoF nie ma własnych opcji.
launch-cvars-from-release = Pochodzą z zainstalowanej wersji ReCoF.
launch-cvars-defaults = Przywróć wartości domyślne

## Aktualizacje

update-title = Aktualizacja
update-available-title = Dostępny jest ReCoF { $version }
update-installed-version = Masz wersję { $version }
update-changelog = Co się zmieniło
update-no-changelog = Ta wersja nie ma opisu zmian.
update-install = Zaktualizuj teraz
update-later = Później
update-downloading = Pobieranie { $name }
update-verifying = Sprawdzanie podpisu i sum kontrolnych
update-extracting = Rozpakowywanie
update-applying = Stosowanie aktualizacji
update-done = Zaktualizowano do ReCoF { $version }.
update-backup-first = Najpierw tworzona jest kopia zapasowa zapisów
update-launcher-first = Ta wersja ReCoF wymaga programu w wersji { $needed } lub nowszej. Najpierw zaktualizuj program.
update-launcher-title = Aktualizacja programu
update-launcher-restart = Program uruchomi się ponownie, żeby dokończyć aktualizację.
update-from-file = Zainstaluj z pliku
update-from-file-help = Wskaż archiwum zip wydania. Podpisany manifest musi leżeć obok.
changelog-note = Uwaga
changelog-tip = Wskazówka
changelog-important = Ważne
changelog-warning = Ostrzeżenie
changelog-caution = Ostrożnie

channel-title = Kanał wydań
channel-stable = Stabilny
channel-beta = Beta
channel-beta-help = Kanał beta zawiera wersje przedpremierowe. Spodziewaj się problemów i zachowaj kopie zapasowe.

## Cofanie się

rollback-title = Wróć do wcześniejszej wersji
rollback-none = Nie zachowano żadnej wcześniejszej wersji.
rollback-to = Wróć do { $version }
rollback-offer-backup = Przywróć zapisy zabezpieczone przed tamtą aktualizacją
rollback-done = Przywrócono ReCoF { $version }.

## Konserwacja

maintain-title = Konserwacja
maintain-verify = Sprawdź i napraw
maintain-verify-help = Liczy sumę kontrolną każdego pliku ReCoF i zapisuje na nowo to, co się nie zgadza.
maintain-backup = Kopia zapasowa zapisów
maintain-restore = Przywróć zapisy
maintain-import = Zaimportuj postępy z kopii ze Steama
maintain-move = Przenieś instalację
maintain-uninstall = Odinstaluj
maintain-shortcuts = Skróty
maintain-logs = Dzienniki

verify-running = Sprawdzanie { $done } z { $total }
verify-intact = Wszystkie pliki się zgadzają.
verify-broken = { $count } plików się nie zgadza.
verify-repair = Napraw je
verify-repaired = Zapisano na nowo { $count } plików.
verify-cannot-repair = Wersji, z której pochodzą te pliki, już nie ma w magazynie. Zainstaluj ponownie albo zaktualizuj, żeby naprawić.

## Kopie zapasowe zapisów

backup-title = Kopie zapasowe zapisów
backup-what = Zapisywane są: zapisy gry, ustawienia, odblokowane rzeczy i twoje pliki konfiguracyjne.
backup-create = Utwórz kopię teraz
backup-created = Zapisano w { $name }
backup-none = Nie ma jeszcze żadnej kopii zapasowej.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = Przywrócić { $name }? Zapisy w twojej kopii ReCoF zostaną zastąpione.
backup-restored = Kopia zapasowa została przywrócona.
backup-verify-failed = Nie udało się odczytać kopii zapasowej po jej zapisaniu, więc nie można jej ufać. Nic nie zostało zmienione.
backup-auto = Utworzona automatycznie przed aktualizacją do { $version }

## Import z kopii ze Steama

import-title = Importuj postępy
import-explain = Kopiuje zapisy, ustawienia i odblokowane rzeczy z instalacji ze Steama do kopii ReCoF. Do instalacji ze Steama nic nie jest zapisywane, a w drugą stronę nic nigdy nie jest kopiowane.
import-nothing-found = W kopii ze Steama nie ma zapisów do zaimportowania.
import-overwrite-title = To zastąpi pliki, które już masz
import-overwrite-body = Zostanie zastąpionych { $count } plików w kopii ReCoF. Wcześniej tworzona jest kopia zapasowa.
import-done = Zaimportowano { $count } plików.

## Przenoszenie

move-title = Przenieś instalację
move-to = Przenieś do
move-running = Przenoszenie
move-done = Instalacja znajduje się teraz w { $path }.
move-same-volume = Na tym samym dysku dzieje się to natychmiast.

## Odinstalowanie

uninstall-title = Odinstaluj ReCoF
uninstall-explain = To usuwa kopię ReCoF i zachowane wersje. Twoja instalacja ze Steama nie jest ruszana.
uninstall-export-first = Najpierw wyeksportuj moje zapisy
uninstall-keep-backups = Zachowaj moje kopie zapasowe zapisów
uninstall-confirm = Usunąć { $path }?
uninstall-done = ReCoF został usunięty.

## Skróty

shortcut-desktop = Skrót na pulpicie
shortcut-start-menu = Skrót w menu Start
shortcut-steam = Dodaj ReCoF do Steama jako grę spoza Steama
shortcut-steam-help = To jedyna rzecz, jaką program zapisuje poza własnymi folderami. Steam musi być zamknięty, a wcześniej tworzona jest kopia pliku shortcuts.vdf.
shortcut-steam-needs-steam-closed = Najpierw zamknij Steama. Steam zapisuje ten plik od nowa przy wyjściu, co cofnęłoby zmianę.
shortcut-steam-added = ReCoF został dodany do Steama. Uruchom Steama ponownie, żeby go zobaczyć.
shortcut-created = Skrót utworzony.

## Zgłaszanie błędu

bug-title = Zgłoś błąd
bug-explain = Zbierane są wersje, wynik ostatniego sprawdzenia, dane twojego systemu i ostatnie linie dziennika. Przejrzyj to, usuń, czego wolisz nie udostępniać, a program otworzy w przeglądarce wstępnie wypełnione zgłoszenie.
bug-never-sends = Nic nigdzie nie jest wysyłane. Program tylko otwiera przeglądarkę.
bug-collecting = Zbieranie danych
bug-open-issue = Otwórz zgłoszenie w przeglądarce
bug-copied = Skopiowano do schowka.

## Ustawienia

settings-title = Ustawienia
settings-language = Język
settings-check-updates = Sprawdzaj aktualizacje przy uruchomieniu programu
settings-discord = Discord Rich Presence
settings-discord-help = Pokazuje w twoim profilu Discorda, że grasz w ReCoF, dopóki gra jest otwarta. Domyślnie wyłączone.
settings-keep-versions = Liczba wersji zachowywanych na wypadek cofnięcia
settings-open-logs = Otwórz folder dzienników
settings-about = O programie

## Odzyskiwanie po przerwanej operacji

recovery-rolled-back = Przerwana operacja została cofnięta. Instalacja jest taka, jaka była przed jej rozpoczęciem.
recovery-rolled-forward = Przerwana operacja została dokończona. Instalacja jest w wersji { $version }.

## Błędy

err-io = Nie udało się odczytać ani zapisać pliku: { $path }
err-not-supported = To nie jest dostępne w tym systemie.
err-registry = Nie udało się odczytać klucza rejestru Windows { $key }.
err-platform-api = { $api } zakończyło się niepowodzeniem.
err-no-game-exe = W { $path } nie ma cof.exe, więc to nie jest folder Cry of Fear.
err-no-game-dir = W { $path } nie ma folderu cryoffear, więc to nie jest folder Cry of Fear.
err-journal = Nie udało się dokończyć operacji instalacji. Nic nie zostaje w połowie: przy następnym uruchomieniu program ją dokończy albo cofnie.
err-interrupted = Operacja została zatrzymana.

err-no-trusted-keys = Ta kompilacja programu nie ma klucza podpisu, więc nie może zweryfikować żadnego wydania. Nic nie zostało zainstalowane.
err-bad-signature = Podpis tego wydania się nie zgadza. Nic nie zostało zainstalowane. Pobierz je ponownie, a jeśli dalej się nie udaje, zgłoś to.
err-bad-manifest = To wydanie opisuje samo siebie w sposób, którego program nie przyjmuje. Nic nie zostało zainstalowane.
err-launcher-too-old = To wydanie wymaga nowszego programu. Najpierw zaktualizuj program.
err-wrong-build = To wydanie zbudowano pod inną wersję Cry of Fear.
err-hash-mismatch = Pobrany plik nie jest tym, czym powinien być. Nic nie zostało zainstalowane.

err-network = Nie udało się połączyć z GitHubem. Sprawdź połączenie i spróbuj ponownie.
err-github = GitHub odpowiedział błędem: { $detail }
err-rate-limited = GitHub ogranicza liczbę zapytań z tego połączenia. Spróbuj za kilka minut.
err-no-release = Na tym kanale nie opublikowano jeszcze żadnego wydania.
err-command-line-too-long = Wiersz poleceń miałby { $length } znaków, a gra przyjmuje najwyżej { $limit }. Skróć dodatkowe argumenty.

## Ustawienia, w zakładkach

settings-tab-look = Wygląd
settings-tab-language = Język
settings-tab-updates = Aktualizacje
settings-tab-behaviour = Zachowanie
settings-tab-about = O programie

## Wygląd

look-title = Wygląd
look-show-hero = Używaj obrazu tła
look-hero-dim = Jak mocno przyciemniane jest tło
look-hero-dim-help = Na tyle, żeby tekst na nim dało się czytać.
look-grain = Ziarno filmowe
look-command-line = Pokazuj wiersz poleceń na ekranie głównym

look-artwork-title = Twoje własne grafiki
look-artwork-help = Program nie zawiera żadnych obrazów z gry. Włóż swoje do tego folderu, a je wykorzysta: hero.png jako tło, logo.png jako tytuł, icon.png jako ikonę okna. Zostają na tym komputerze i nigdy nie trafiają do wydania.
look-open-theme = Otwórz folder grafik
look-reload-art = Wczytaj grafiki ponownie
look-using-art = Używane są twoje grafiki: { $pieces }
look-no-art = Nie znaleziono grafik, więc program rysuje własne.

## Zachowanie

behaviour-minimize = Zminimalizuj program po uruchomieniu gry
behaviour-close = Zamknij program po uruchomieniu gry
behaviour-close-help = Ma pierwszeństwo przed minimalizowaniem.

## O programie

about-what = ReCoF Launcher instaluje, aktualizuje i uruchamia ReCoF, ulepszoną rekompilację Cry of Fear. Kopiuje twoją instalację ze Steama do wskazanego folderu i nigdy nie zapisuje niczego w tej ze Steama.
about-made-by = Autor: { $author }, który tworzy też samo ReCoF.
about-version = Program { $version }
about-licence = Licencja MIT. Kod jest pod { $url }.
about-recof = ReCoF jest pod { $url }.
about-font = Tytuł złożono krojem Archivo Black, na licencji SIL Open Font License 1.1.
about-not-affiliated = Bez związku z Team Psykskallar. Cry of Fear należy do swoich autorów; ten program instaluje jedynie rekompilację zrobioną z twojej własnej kopii.

## Źródła grafik

look-credits-title = Źródła grafik
look-credits-none = Te grafiki przyszły bez informacji o autorach.
look-credits-help = Grafiki nie są częścią programu. Pochodzą z wydania ReCoF i należą do tych, którzy je zrobili.
look-forget-art = Usuń grafiki
nav-home = Start
nav-options = Opcje uruchamiania
nav-maintain = Konserwacja
nav-bug = Zgłoś problem
nav-settings = Ustawienia
nav-folder = Otwórz folder instalacji
home-more = Więcej
home-stat-last-played = Ostatnio grano
home-stat-playtime = Łączny czas gry
home-stat-version = Zainstalowana wersja
home-played-never = Nigdy
home-played-today = Dzisiaj
home-played-yesterday = Wczoraj
home-played-days = { $days } dni temu
home-playtime-none = Jeszcze nic
home-playtime-minutes = { $minutes } min
home-playtime-hours = { $hours } godz { $minutes } min
settings-discord-unavailable = Ta kompilacja nie ma identyfikatora aplikacji Discord, więc status nie może być pokazany.
