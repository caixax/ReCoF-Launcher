# ReCoF Launcher, italiano.
#
# PRIMA BOZZA: questa traduzione non è ancora stata rivista da una persona
# madrelingua. Le correzioni sono benvenute:
# https://github.com/caixax/ReCoF-Launcher
#
# Stile: frasi semplici, nessuna lineetta lunga o media.

app-name = ReCoF Launcher
app-tagline = Installatore e avviatore di ReCoF

## Pulsanti e parole che compaiono ovunque

button-next = Avanti
button-back = Indietro
button-cancel = Annulla
button-close = Chiudi
button-continue = Continua
button-ok = OK
button-yes = Sì
button-no = No
button-browse = Sfoglia...
button-retry = Riprova
button-copy = Copia
button-open-folder = Apri cartella
button-finish = Fine
button-skip = Salta
button-details = Dettagli
word-yes = sì
word-no = no
word-unknown = sconosciuto

## Primo avvio: lingua

firstrun-language-title = Scegli la lingua
firstrun-language-help = Puoi cambiarla più avanti nelle impostazioni.
language-not-reviewed = Prima bozza, non ancora rivista da una persona madrelingua
language-not-reviewed-notice = Questa traduzione è una prima bozza e non è ancora stata rivista da una persona madrelingua. Le correzioni sono benvenute.

## Primo avvio: di cosa si tratta

intro-1-title = Cos'è ReCoF
intro-1-body = ReCoF è una ricompilazione migliorata di Cry of Fear. Questo avviatore lo installa, lo tiene aggiornato e lo lancia. ReCoF e questo avviatore sono progetti distinti dal gioco originale.
intro-2-title = La tua copia Steam non viene mai toccata
intro-2-body = L'avviatore legge la tua installazione Steam e la copia in una cartella che scegli tu. Non scrive, non cancella e non rinomina mai nulla dentro la cartella di Steam, quindi Cry of Fear continua a funzionare esattamente come adesso e puoi tornarci quando vuoi.
intro-3-title = Una copia a parte, e i tuoi progressi
intro-3-body = Dato che ReCoF vive nella sua copia, gli serve spazio per tutto il gioco, circa { $size }. I salvataggi, le impostazioni e gli sblocchi si possono portare dalla copia Steam durante l'installazione, oppure più avanti dal menu. Nulla viene mai copiato nella direzione opposta.

## Trovare il gioco

detect-title = Ricerca di Cry of Fear
detect-searching = Ricerca dell'installazione di Steam...
detect-found = Trovato in { $path }
detect-not-found = Cry of Fear non è stato trovato
detect-not-found-help = Forse Steam non è installato, o il gioco si trova in una cartella che l'avviatore non vede. Scegli la cartella che contiene cof.exe.
detect-pick-folder = Scegli la cartella del gioco
detect-pick-folder-help = La cartella che contiene cof.exe
detect-library = Libreria Steam: { $path }
detect-buildid-ok = Build { $buildid }, quella per cui ReCoF è compilato
detect-buildid-wrong = Questa è la build { $found }. ReCoF è compilato per la build { $expected }. Potrebbe non funzionare bene, ed è lecito aspettarsi problemi.
detect-buildid-unknown = Non è stato possibile leggere la build di Steam. È normale per una cartella scelta a mano.
detect-size = { $files } file, { $size }

## Cosa sono risultati i file

detect-check-title = Controllo dei file del gioco
detect-check-clean = Tutti i file corrispondono alla versione commerciale 1.6a.
detect-check-notices = Alcuni file non sono quelli della versione commerciale 1.6a. Questo non blocca l'installazione: la copia è tua e ReCoF sostituisce comunque quei file. È indicato qui perché tu sappia cosa stai copiando.
verdict-retail = corrisponde alla versione commerciale
verdict-patched = versione commerciale, con la patch da 4 GB applicata
verdict-unknown = non è un file che l'avviatore conosce, SHA-256 { $sha256 }
verdict-missing = mancante
detect-unexpected-title = Programmi in più nella cartella del gioco
detect-unexpected-help = La versione commerciale di Cry of Fear non li include. Di solito arrivano da una patch della comunità.
detect-mods-title = Modifiche note

variant-cof-enhanced = la versione installata dalla patch Cry of Fear Enhanced
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = sostituzione del motore con Xash3D
mod-xash-menu = menu di Xash3D
mod-bunnymod-xt = Bunnymod XT

## Dove installare

install-title = Installa ReCoF
install-destination = Installa in
install-destination-help = Una cartella nuova, fuori da Steam. ReCoF ha bisogno di una copia completa del gioco.
install-size-line = La copia richiede { $size }. Su { $volume } sono liberi { $available }.
install-space-ok = C'è spazio.
install-space-short = Non c'è spazio sufficiente su { $volume }. Mancano { $missing }.
install-space-explain = Oltre alla copia, l'avviatore riserva spazio per le ultime versioni e per i backup dei salvataggi, così puoi tornare indietro e un aggiornamento non comincia mai senza via d'uscita.

dest-inside-steam = Quella cartella è dentro la cartella di Steam. ReCoF deve stare fuori, perché un aggiornamento di Steam non possa mai sovrascriverlo.
dest-inside-steamapps = Quella cartella è dentro lo steamapps di una libreria Steam, dove i file li gestisce Steam. Scegli un altro posto.
dest-protected = Quella cartella richiede i permessi di amministratore e non è un buon posto per un gioco. Scegli qualcosa tra i tuoi file.
dest-is-vanilla = Quella è la copia Steam del gioco. ReCoF va in una cartella a parte.
dest-drive-root = Quella è la radice di un disco. Scegli una cartella al suo interno.
dest-not-empty = Quella cartella contiene già { $entries } file di altro. Scegli una cartella vuota o una nuova.
dest-unusable = Quel percorso non si può usare.

## Installazione

install-copying = Copia del gioco: { $current }
install-copying-counted = Copia del gioco, { $done } di { $total }
install-applying = Applicazione di ReCoF { $version }
install-finishing = Completamento
install-done = ReCoF { $version } è installato.
install-cancel-confirm = Fermare l'installazione? Verrà rimosso tutto quello che è stato copiato finora. La tua copia Steam non viene toccata.
install-cancelled = L'installazione è stata annullata e non ha lasciato nulla a metà.
install-offer-import = Portare salvataggi, impostazioni e sblocchi dalla copia Steam
install-offer-shortcuts = Creare un collegamento sul desktop
install-release-title = Cosa viene installato
install-release-version = ReCoF { $version }
install-release-download = Viene scaricato da GitHub e verificato con la sua firma prima di copiare qualsiasi cosa.
install-release-stored = Già scaricato e verificato.
install-release-file = Da { $name }
install-release-file-help = Il manifesto firmato accanto allo zip viene verificato prima di copiare qualsiasi cosa.
install-release-file-clear = Usa invece una versione pubblicata
install-release-checking = Ricerca dell'ultima versione
install-release-offline = Impossibile raggiungere GitHub. Riprova o installa da un file.
install-release-try-beta = Sul canale stabile non è ancora stato pubblicato nulla. I tester possono passare alla beta.

## La schermata principale

home-play = Gioca a ReCoF
home-play-vanilla = Gioca all'originale
home-no-install = ReCoF non è ancora installato
home-install-now = Installa ReCoF
home-checking = Ricerca di aggiornamenti...
home-up-to-date = Tutto aggiornato
home-update-available = È disponibile ReCoF { $version }
home-launcher-update-available = È disponibile una nuova versione dell'avviatore: { $version }
home-offline = Non è stato possibile raggiungere GitHub. L'avviatore funziona lo stesso.

## Avviare il gioco

launch-steam-not-running = Steam non è in esecuzione
launch-steam-not-running-help = Cry of Fear usa Steam mentre gira. Avvia prima Steam.
launch-open-steam = Apri Steam
launch-start-anyway = Avvia comunque
launch-starting = Avvio di ReCoF...
launch-failed = Non è stato possibile avviare ReCoF.
launch-already-running = ReCoF è già in esecuzione.

launch-options-title = Opzioni di avvio
launch-resolution = Risoluzione
launch-resolution-default = Quella che decide il gioco
launch-windowed = In finestra
launch-widescreen = Schermo panoramico
launch-3dmenu = Menu 3D
launch-3dmenu-help = La versione commerciale parte con il menu 3D disattivato. Attivandolo si usa il menu animato originale.
launch-extra-args = Argomenti aggiuntivi
launch-extra-args-help = Vengono passati a cof.exe così come sono. Lascia vuoto se non sai cosa ti serve.
launch-preview = Riga di comando

launch-cvars-title = Opzioni di ReCoF
launch-cvars-none = Questa versione di ReCoF non porta opzioni proprie.
launch-cvars-from-release = Arrivano dalla versione di ReCoF che hai installato.
launch-cvars-defaults = Ripristina i valori predefiniti

## Aggiornamenti

update-title = Aggiorna
update-available-title = È disponibile ReCoF { $version }
update-installed-version = Hai la { $version }
update-changelog = Cosa è cambiato
update-no-changelog = Questa versione non ha note.
update-install = Aggiorna ora
update-later = Più tardi
update-downloading = Scaricamento di { $name }
update-verifying = Controllo della firma e delle impronte
update-extracting = Estrazione
update-applying = Applicazione dell'aggiornamento
update-done = Aggiornato a ReCoF { $version }.
update-backup-first = Prima viene fatto un backup dei tuoi salvataggi
update-launcher-first = Questa versione di ReCoF richiede l'avviatore { $needed } o più recente. Aggiorna prima l'avviatore.
update-launcher-title = Aggiornamento dell'avviatore
update-launcher-restart = L'avviatore si riavvierà per completare l'aggiornamento.
update-from-file = Installa da un file
update-from-file-help = Scegli un archivio zip di una versione. Il suo manifesto firmato deve stare accanto.
changelog-note = Nota
changelog-tip = Suggerimento
changelog-important = Importante
changelog-warning = Avviso
changelog-caution = Attenzione

channel-title = Canale delle versioni
channel-stable = Stabile
channel-beta = Beta
channel-beta-help = Il canale beta include le anteprime. Aspettati problemi e tieni i tuoi backup.

## Tornare indietro

rollback-title = Torna a una versione precedente
rollback-none = Non è conservata nessuna versione precedente.
rollback-to = Torna a { $version }
rollback-offer-backup = Ripristina i salvataggi messi da parte prima di quell'aggiornamento
rollback-done = ReCoF { $version } è stato ripristinato.

## Manutenzione

maintain-title = Manutenzione
maintain-verify = Verifica e ripara
maintain-verify-help = Calcola l'impronta di ogni file di ReCoF e riscrive tutto quello che non corrisponde.
maintain-backup = Backup dei salvataggi
maintain-restore = Ripristina i salvataggi
maintain-import = Importa i progressi dalla copia Steam
maintain-move = Sposta l'installazione
maintain-uninstall = Disinstalla
maintain-shortcuts = Collegamenti
maintain-logs = Registri

verify-running = Controllo di { $done } su { $total }
verify-intact = Tutti i file corrispondono.
verify-broken = { $count } file non corrispondono.
verify-repair = Riparali
verify-repaired = { $count } file sono stati riscritti.
verify-cannot-repair = La versione da cui provengono questi file non è più nell'archivio. Reinstalla o aggiorna per riparare.

## Backup dei salvataggi

backup-title = Backup dei salvataggi
backup-what = Vengono salvati: partite, impostazioni, sblocchi e i tuoi file di configurazione.
backup-create = Fai il backup ora
backup-created = Salvato in { $name }
backup-none = Non c'è ancora nessun backup.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = Ripristinare { $name }? I salvataggi della tua copia ReCoF verranno sostituiti.
backup-restored = Il backup è stato ripristinato.
backup-verify-failed = Il backup non si è potuto rileggere dopo averlo scritto, quindi non è affidabile. Non è stato cambiato nulla.
backup-auto = Fatto automaticamente prima di aggiornare a { $version }

## Importare dalla copia Steam

import-title = Importa i progressi
import-explain = Copia salvataggi, impostazioni e sblocchi dall'installazione Steam nella copia ReCoF. Nell'installazione Steam non viene scritto nulla, e nulla viene mai copiato nella direzione opposta.
import-nothing-found = La copia Steam non ha salvataggi da importare.
import-overwrite-title = Questo sostituirà file che hai già
import-overwrite-body = Verranno sostituiti { $count } file nella copia ReCoF. Prima viene fatto un backup.
import-done = Sono stati importati { $count } file.

## Spostare

move-title = Sposta l'installazione
move-to = Sposta in
move-running = Spostamento
move-done = L'installazione si trova ora in { $path }.
move-same-volume = Sullo stesso disco è immediato.

## Disinstallare

uninstall-title = Disinstalla ReCoF
uninstall-explain = Questo rimuove la copia di ReCoF e le versioni conservate. La tua installazione Steam non viene toccata.
uninstall-export-first = Esporta prima i miei salvataggi
uninstall-keep-backups = Conserva i miei backup dei salvataggi
uninstall-confirm = Rimuovere { $path }?
uninstall-done = ReCoF è stato rimosso.

## Collegamenti

shortcut-desktop = Collegamento sul desktop
shortcut-start-menu = Collegamento nel menu Start
shortcut-steam = Aggiungi ReCoF a Steam come gioco non Steam
shortcut-steam-help = È l'unica cosa che l'avviatore scrive fuori dalle proprie cartelle. Steam deve essere chiuso, e prima viene fatta una copia di shortcuts.vdf.
shortcut-steam-needs-steam-closed = Chiudi prima Steam. Steam riscrive quel file quando esce, annullando la modifica.
shortcut-steam-added = ReCoF è stato aggiunto a Steam. Riavvia Steam per vederlo.
shortcut-created = Collegamento creato.

## Segnalare un problema

bug-title = Segnala un problema
bug-explain = Questo raccoglie le versioni, il risultato dell'ultima verifica, i dati del tuo sistema e le ultime righe del registro. Rileggilo, togli quello che preferisci non condividere, e l'avviatore aprirà nel browser una segnalazione già compilata.
bug-never-sends = Non viene inviato nulla da nessuna parte. L'avviatore apre solo il tuo browser.
bug-collecting = Raccolta dei dati
bug-open-issue = Apri la segnalazione nel browser
bug-copied = Copiato negli appunti.

## Impostazioni

settings-title = Impostazioni
settings-language = Lingua
settings-check-updates = Cerca aggiornamenti all'avvio dell'avviatore
settings-discord = Discord Rich Presence
settings-discord-help = Mostra sul tuo profilo Discord che stai giocando a ReCoF, mentre il gioco è aperto. Disattivato in partenza.
settings-keep-versions = Versioni da conservare per tornare indietro
settings-open-logs = Apri la cartella dei registri
settings-about = Informazioni

## Recupero dopo un'operazione interrotta

recovery-rolled-back = Un'operazione interrotta è stata annullata. L'installazione è come era prima che cominciasse.
recovery-rolled-forward = Un'operazione interrotta è stata completata. L'installazione è alla versione { $version }.

## Errori

err-io = Non è stato possibile leggere o scrivere un file: { $path }
err-not-supported = Questo non è disponibile su questo sistema.
err-registry = Non è stato possibile leggere la chiave di registro di Windows { $key }.
err-platform-api = { $api } non è riuscita.
err-no-game-exe = { $path } non contiene cof.exe, quindi non è una cartella di Cry of Fear.
err-no-game-dir = { $path } non contiene una cartella cryoffear, quindi non è una cartella di Cry of Fear.
err-journal = Non è stato possibile completare un'operazione di installazione. Non resta nulla a metà: l'avviatore la completerà o la annullerà al prossimo avvio.
err-interrupted = L'operazione è stata fermata.

err-no-trusted-keys = Questa compilazione dell'avviatore non ha una chiave di firma, quindi non può verificare nessuna versione. Non è stato installato nulla.
err-bad-signature = La firma di questa versione non torna. Non è stato installato nulla. Scaricala di nuovo, e se continua a fallire, segnalalo.
err-bad-manifest = Questa versione si descrive in un modo che l'avviatore non accetta. Non è stato installato nulla.
err-launcher-too-old = Questa versione richiede un avviatore più recente. Aggiorna prima l'avviatore.
err-wrong-build = Questa versione è compilata per un'altra versione di Cry of Fear.
err-hash-mismatch = Un file scaricato non è quello che dovrebbe essere. Non è stato installato nulla.

err-network = Non è stato possibile raggiungere GitHub. Controlla la connessione e riprova.
err-github = GitHub ha risposto con un errore: { $detail }
err-rate-limited = GitHub sta limitando le richieste di questa connessione. Riprova tra qualche minuto.
err-no-release = Su questo canale non è ancora stata pubblicata nessuna versione.
err-command-line-too-long = La riga di comando sarebbe lunga { $length } caratteri e il gioco ne accetta al massimo { $limit }. Accorcia gli argomenti aggiuntivi.

## Impostazioni, a schede

settings-tab-look = Aspetto
settings-tab-language = Lingua
settings-tab-updates = Aggiornamenti
settings-tab-behaviour = Comportamento
settings-tab-about = Informazioni

## Aspetto

look-title = Aspetto
look-show-hero = Usa l'immagine di sfondo
look-hero-dim = Quanto viene scurito lo sfondo
look-hero-dim-help = Abbastanza perché il testo sopra resti leggibile.
look-grain = Grana della pellicola
look-command-line = Mostra la riga di comando nella schermata principale

look-artwork-title = Le tue immagini
look-artwork-help = L'avviatore non porta con sé nessuna immagine del gioco. Metti le tue in questa cartella e le userà: hero.png per lo sfondo, logo.png per il titolo, icon.png per l'icona della finestra. Restano su questa macchina e non finiscono mai in una pubblicazione.
look-open-theme = Apri la cartella delle immagini
look-reload-art = Ricarica le immagini
look-using-art = Sto usando le tue immagini: { $pieces }
look-no-art = Nessuna immagine trovata, quindi l'avviatore disegna le proprie.

## Comportamento

behaviour-minimize = Riduci a icona l'avviatore quando parte il gioco
behaviour-close = Chiudi l'avviatore quando parte il gioco
behaviour-close-help = Ha la precedenza sulla riduzione a icona.

## Informazioni

about-what = ReCoF Launcher installa, aggiorna e avvia ReCoF, la ricompilazione migliorata di Cry of Fear. Copia la tua installazione di Steam in una cartella che scegli tu e non scrive mai dentro quella di Steam.
about-made-by = Fatto da { $author }, che fa anche ReCoF.
about-version = Avviatore { $version }
about-licence = Licenza MIT. Il codice è su { $url }.
about-recof = ReCoF è su { $url }.
about-font = Il titolo è composto in Archivo Black, con licenza SIL Open Font License 1.1.
about-not-affiliated = Nessun legame con Team Psykskallar. Cry of Fear appartiene ai suoi autori; questo avviatore installa soltanto una ricompilazione fatta dalla tua copia.

## Crediti delle immagini

look-credits-title = Crediti delle immagini
look-credits-none = Queste immagini sono arrivate senza crediti.
look-credits-help = Le immagini non fanno parte dell'avviatore. Vengono dalla pubblicazione di ReCoF e appartengono a chi le ha fatte.
look-forget-art = Togli le immagini
nav-home = Inizio
nav-options = Opzioni di avvio
nav-maintain = Manutenzione
nav-bug = Segnala un problema
nav-settings = Impostazioni
nav-folder = Apri la cartella di installazione
home-more = Altro
home-stat-last-played = Ultima partita
home-stat-playtime = Tempo di gioco totale
home-stat-version = Versione installata
home-played-never = Mai
home-played-today = Oggi
home-played-yesterday = Ieri
home-played-days = { $days } giorni fa
home-playtime-none = Ancora niente
home-playtime-minutes = { $minutes } min
home-playtime-hours = { $hours } h { $minutes } min
settings-discord-unavailable = Questa build non ha un id applicazione Discord, quindi la presenza non può essere mostrata.
