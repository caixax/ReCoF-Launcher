# ReCoF Launcher, français.
#
# PREMIER JET : cette traduction n'a pas encore été relue par une personne
# de langue maternelle française. Les corrections sont les bienvenues :
# https://github.com/caixax/ReCoF-Launcher
#
# Style : phrases simples, aucun tiret cadratin ni demi-cadratin.

app-name = ReCoF Launcher
app-tagline = Installateur et lanceur de ReCoF

## Boutons et mots qui reviennent partout

button-next = Suivant
button-back = Retour
button-cancel = Annuler
button-close = Fermer
button-continue = Continuer
button-ok = OK
button-yes = Oui
button-no = Non
button-browse = Parcourir...
button-retry = Réessayer
button-copy = Copier
button-open-folder = Ouvrir le dossier
button-finish = Terminer
button-skip = Passer
button-details = Détails
word-yes = oui
word-no = non
word-unknown = inconnu

## Premier démarrage : langue

firstrun-language-title = Choisissez votre langue
firstrun-language-help = Vous pourrez changer cela plus tard dans les réglages.
language-not-reviewed = Premier jet, pas encore relu par une personne de langue maternelle
language-not-reviewed-notice = Cette traduction est un premier jet et n'a pas encore été relue par une personne de langue maternelle. Les corrections sont les bienvenues.

## Premier démarrage : de quoi il s'agit

intro-1-title = Ce qu'est ReCoF
intro-1-body = ReCoF est une recompilation améliorée de Cry of Fear. Ce lanceur l'installe, le tient à jour et le démarre. ReCoF et ce lanceur sont des projets distincts du jeu lui-même.
intro-2-title = Votre copie Steam n'est jamais touchée
intro-2-body = Le lanceur lit votre installation Steam et la copie dans un dossier que vous choisissez. Il n'écrit, ne supprime et ne renomme jamais rien dans le dossier Steam, donc Cry of Fear continue de fonctionner exactement comme maintenant et vous pouvez y revenir quand vous voulez.
intro-3-title = Une copie à part, et votre progression
intro-3-body = Comme ReCoF vit dans sa propre copie, il lui faut la place du jeu entier, environ { $size }. Vos sauvegardes, réglages et déblocages peuvent être repris depuis la copie Steam pendant l'installation, ou plus tard depuis le menu. Rien n'est jamais copié dans l'autre sens.

## Trouver le jeu

detect-title = Recherche de Cry of Fear
detect-searching = Recherche de votre installation Steam...
detect-found = Trouvé dans { $path }
detect-not-found = Cry of Fear est introuvable
detect-not-found-help = Steam n'est peut-être pas installé, ou le jeu se trouve dans un dossier que le lanceur ne voit pas. Choisissez le dossier qui contient cof.exe.
detect-pick-folder = Choisir le dossier du jeu
detect-pick-folder-help = Le dossier qui contient cof.exe
detect-library = Bibliothèque Steam : { $path }
detect-buildid-ok = Build { $buildid }, celle pour laquelle ReCoF est compilé
detect-buildid-wrong = Il s'agit de la build { $found }. ReCoF est compilé pour la build { $expected }. Il risque de ne pas fonctionner correctement, et il faut s'attendre à des problèmes.
detect-buildid-unknown = La build Steam n'a pas pu être lue. C'est normal pour un dossier choisi à la main.
detect-size = { $files } fichiers, { $size }

## Ce que sont les fichiers

detect-check-title = Vérification des fichiers du jeu
detect-check-clean = Tous les fichiers correspondent à la version commerciale 1.6a.
detect-check-notices = Certains fichiers ne sont pas ceux de la version commerciale 1.6a. Cela n'empêche pas l'installation : la copie est la vôtre et ReCoF remplace ces fichiers de toute façon. C'est indiqué ici pour que vous sachiez ce que vous copiez.
verdict-retail = correspond à la version commerciale
verdict-patched = version commerciale, avec le correctif 4 Go appliqué
verdict-unknown = fichier inconnu du lanceur, SHA-256 { $sha256 }
verdict-missing = absent
detect-unexpected-title = Programmes en trop dans le dossier du jeu
detect-unexpected-help = La version commerciale de Cry of Fear ne les fournit pas. Ils viennent généralement d'un correctif communautaire.
detect-mods-title = Modifications connues

variant-cof-enhanced = la version installée par le correctif Cry of Fear Enhanced
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = remplacement du moteur par Xash3D
mod-xash-menu = menu Xash3D
mod-bunnymod-xt = Bunnymod XT

## Où installer

install-title = Installer ReCoF
install-destination = Installer dans
install-destination-help = Un nouveau dossier, en dehors de Steam. ReCoF a besoin d'une copie complète du jeu.
install-size-line = La copie demande { $size }. Il reste { $available } sur { $volume }.
install-space-ok = Il y a la place.
install-space-short = Il n'y a pas assez de place sur { $volume }. Il manque { $missing }.
install-space-explain = En plus de la copie, le lanceur réserve de la place pour les dernières versions et pour les sauvegardes, afin que vous puissiez revenir en arrière et qu'une mise à jour ne commence jamais sans issue.

dest-inside-steam = Ce dossier est à l'intérieur du dossier Steam. ReCoF doit vivre en dehors, pour qu'une mise à jour de Steam ne puisse jamais l'écraser.
dest-inside-steamapps = Ce dossier est dans le steamapps d'une bibliothèque Steam, là où Steam gère les fichiers. Choisissez un autre endroit.
dest-protected = Ce dossier demande des droits d'administrateur et n'est pas un bon endroit pour un jeu. Choisissez quelque chose dans vos propres fichiers.
dest-is-vanilla = C'est la copie Steam du jeu. ReCoF va dans un dossier à part.
dest-drive-root = C'est la racine d'un disque. Choisissez un dossier à l'intérieur.
dest-not-empty = Ce dossier contient déjà { $entries } autres fichiers. Choisissez un dossier vide ou un nouveau.
dest-unusable = Ce chemin ne peut pas être utilisé.

## Installation

install-copying = Copie du jeu : { $current }
install-copying-counted = Copie du jeu, { $done } sur { $total }
install-applying = Application de ReCoF { $version }
install-finishing = Finalisation
install-done = ReCoF { $version } est installé.
install-cancel-confirm = Arrêter l'installation ? Tout ce qui a été copié sera supprimé. Votre copie Steam n'est pas touchée.
install-cancelled = L'installation a été annulée et n'a rien laissé derrière elle.
install-offer-import = Reprendre mes sauvegardes, réglages et déblocages depuis la copie Steam
install-offer-shortcuts = Créer un raccourci sur le bureau

## L'écran principal

home-play = Jouer à ReCoF
home-play-vanilla = Jouer à l'original
home-version-installed = ReCoF { $version } installé
home-no-install = ReCoF n'est pas encore installé
home-install-now = Installer ReCoF
home-channel = Canal : { $channel }
home-checking = Recherche de mises à jour...
home-up-to-date = À jour
home-update-available = ReCoF { $version } est disponible
home-launcher-update-available = Une nouvelle version du lanceur est disponible : { $version }
home-offline = GitHub est injoignable. Le lanceur fonctionne quand même.

## Démarrer le jeu

launch-steam-not-running = Steam n'est pas lancé
launch-steam-not-running-help = Cry of Fear utilise Steam pendant qu'il tourne. Lancez Steam d'abord.
launch-open-steam = Ouvrir Steam
launch-start-anyway = Lancer quand même
launch-starting = Démarrage de ReCoF...
launch-failed = ReCoF n'a pas pu démarrer.
launch-already-running = ReCoF est déjà lancé.

launch-options-title = Options de lancement
launch-resolution = Résolution
launch-resolution-default = Ce que le jeu décide
launch-windowed = Fenêtré
launch-widescreen = Écran large
launch-3dmenu = Menu 3D
launch-3dmenu-help = La version commerciale démarre avec le menu 3D désactivé. En l'activant, le menu animé d'origine est utilisé.
launch-extra-args = Arguments supplémentaires
launch-extra-args-help = Transmis tels quels à cof.exe. Laissez vide si vous ne savez pas ce dont vous avez besoin.
launch-preview = Ligne de commande

launch-cvars-title = Options de ReCoF
launch-cvars-none = Cette version de ReCoF n'apporte aucune option propre.
launch-cvars-from-release = Elles viennent de la version de ReCoF que vous avez installée.

## Mises à jour

update-title = Mise à jour
update-available-title = ReCoF { $version } est disponible
update-installed-version = Vous avez la { $version }
update-changelog = Ce qui a changé
update-no-changelog = Cette version n'a pas de notes.
update-install = Mettre à jour maintenant
update-later = Plus tard
update-downloading = Téléchargement de { $name }
update-verifying = Vérification de la signature et des empreintes
update-extracting = Décompression
update-applying = Application de la mise à jour
update-done = Mis à jour vers ReCoF { $version }.
update-backup-first = Sauvegarde de vos parties au préalable
update-launcher-first = Cette version de ReCoF demande le lanceur { $needed } ou plus récent. Mettez d'abord le lanceur à jour.
update-launcher-title = Mise à jour du lanceur
update-launcher-restart = Le lanceur va redémarrer pour terminer la mise à jour.
update-from-file = Installer depuis un fichier
update-from-file-help = Choisissez une archive de version. Son manifeste signé doit se trouver à côté.

channel-title = Canal de publication
channel-stable = Stable
channel-beta = Bêta
channel-beta-help = Le canal bêta inclut les préversions. Attendez-vous à des problèmes et gardez vos sauvegardes.

## Revenir en arrière

rollback-title = Revenir à une version précédente
rollback-none = Aucune version précédente n'est conservée.
rollback-to = Revenir à { $version }
rollback-offer-backup = Restaurer les sauvegardes faites avant cette mise à jour
rollback-done = ReCoF { $version } a été restauré.

## Maintenance

maintain-title = Maintenance
maintain-verify = Vérifier et réparer
maintain-verify-help = Calculer l'empreinte de chaque fichier de ReCoF et réécrire tout ce qui ne correspond pas.
maintain-backup = Sauvegarder les parties
maintain-restore = Restaurer les parties
maintain-import = Importer la progression depuis la copie Steam
maintain-move = Déplacer l'installation
maintain-uninstall = Désinstaller
maintain-shortcuts = Raccourcis
maintain-logs = Journaux

verify-running = Vérification de { $done } sur { $total }
verify-intact = Tous les fichiers correspondent.
verify-broken = { $count } fichiers ne correspondent pas.
verify-repair = Les réparer
verify-repaired = { $count } fichiers ont été réécrits.
verify-cannot-repair = La version d'où viennent ces fichiers n'est plus dans la réserve. Réinstallez ou mettez à jour pour réparer.

## Sauvegardes des parties

backup-title = Sauvegardes des parties
backup-what = Sont sauvegardés : parties, réglages, déblocages et vos fichiers de configuration.
backup-create = Sauvegarder maintenant
backup-created = Sauvegardé dans { $name }
backup-none = Il n'y a encore aucune sauvegarde.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = Restaurer { $name } ? Les parties de votre copie ReCoF seront remplacées.
backup-restored = La sauvegarde a été restaurée.
backup-verify-failed = La sauvegarde n'a pas pu être relue après son écriture, elle n'est donc pas fiable. Rien n'a été modifié.
backup-auto = Faite automatiquement avant la mise à jour vers { $version }

## Importer depuis la copie Steam

import-title = Importer la progression
import-explain = Copie vos parties, réglages et déblocages depuis l'installation Steam vers la copie ReCoF. Rien n'est écrit dans l'installation Steam, et rien n'est jamais copié dans l'autre sens.
import-nothing-found = La copie Steam n'a aucune partie à importer.
import-overwrite-title = Cela va remplacer des fichiers que vous avez déjà
import-overwrite-body = { $count } fichiers de la copie ReCoF seront remplacés. Une sauvegarde est faite avant.
import-done = { $count } fichiers ont été importés.

## Déplacer

move-title = Déplacer l'installation
move-to = Déplacer vers
move-running = Déplacement
move-done = L'installation se trouve maintenant dans { $path }.
move-same-volume = Sur le même disque, c'est immédiat.

## Désinstaller

uninstall-title = Désinstaller ReCoF
uninstall-explain = Cela supprime la copie ReCoF et les versions conservées. Votre installation Steam n'est pas touchée.
uninstall-export-first = Exporter mes parties d'abord
uninstall-keep-backups = Garder mes sauvegardes de parties
uninstall-confirm = Supprimer { $path } ?
uninstall-done = ReCoF a été supprimé.

## Raccourcis

shortcut-desktop = Raccourci sur le bureau
shortcut-start-menu = Raccourci dans le menu Démarrer
shortcut-steam = Ajouter ReCoF à Steam comme jeu hors Steam
shortcut-steam-help = C'est la seule chose que le lanceur écrit en dehors de ses propres dossiers. Steam doit être fermé, et une copie de shortcuts.vdf est faite avant.
shortcut-steam-needs-steam-closed = Fermez Steam d'abord. Steam réécrit ce fichier en quittant, ce qui annulerait la modification.
shortcut-steam-added = ReCoF a été ajouté à Steam. Relancez Steam pour le voir.
shortcut-created = Raccourci créé.

## Signaler un bogue

bug-title = Signaler un bogue
bug-explain = Ceci rassemble les versions, le résultat de la dernière vérification, les détails de votre système et les dernières lignes du journal. Relisez le tout, retirez ce que vous préférez ne pas partager, et le lanceur ouvrira un rapport prérempli dans votre navigateur.
bug-never-sends = Rien n'est envoyé nulle part. Le lanceur ouvre seulement votre navigateur.
bug-collecting = Collecte en cours
bug-open-issue = Ouvrir le rapport dans mon navigateur
bug-copied = Copié dans le presse-papiers.

## Réglages

settings-title = Réglages
settings-language = Langue
settings-check-updates = Chercher des mises à jour au démarrage du lanceur
settings-discord = Discord Rich Presence
settings-discord-help = Affiche sur votre profil Discord que vous jouez à ReCoF, pendant que le jeu est ouvert. Désactivé par défaut.
settings-keep-versions = Versions conservées pour revenir en arrière
settings-open-logs = Ouvrir le dossier des journaux
settings-about = À propos

## Récupération après une opération interrompue

recovery-rolled-back = Une opération interrompue a été annulée. L'installation est telle qu'elle était avant son début.
recovery-rolled-forward = Une opération interrompue a été terminée. L'installation est en version { $version }.

## Erreurs

err-io = Un fichier n'a pas pu être lu ou écrit : { $path }
err-not-supported = Ceci n'est pas disponible sur ce système.
err-registry = La clé de registre Windows { $key } n'a pas pu être lue.
err-platform-api = { $api } a échoué.
err-no-game-exe = { $path } ne contient pas cof.exe, ce n'est donc pas un dossier de Cry of Fear.
err-no-game-dir = { $path } ne contient pas de dossier cryoffear, ce n'est donc pas un dossier de Cry of Fear.
err-journal = Une opération d'installation n'a pas pu être terminée. Rien n'est à moitié fait : le lanceur la terminera ou l'annulera au prochain démarrage.
err-interrupted = L'opération a été arrêtée.

err-no-trusted-keys = Cette compilation du lanceur n'a pas de clé de signature, elle ne peut donc vérifier aucune version. Rien n'a été installé.
err-bad-signature = La signature de cette version ne correspond pas. Rien n'a été installé. Téléchargez-la à nouveau, et si cela continue d'échouer, signalez-le.
err-bad-manifest = Cette version se décrit d'une manière que le lanceur refuse. Rien n'a été installé.
err-launcher-too-old = Cette version demande un lanceur plus récent. Mettez d'abord le lanceur à jour.
err-wrong-build = Cette version est compilée pour une autre version de Cry of Fear.
err-hash-mismatch = Un fichier téléchargé n'est pas ce qu'il devrait être. Rien n'a été installé.

err-network = GitHub est injoignable. Vérifiez votre connexion et réessayez.
err-github = GitHub a répondu par une erreur : { $detail }
err-rate-limited = GitHub limite les requêtes de cette connexion. Réessayez dans quelques minutes.
err-no-release = Aucune version n'a encore été publiée sur ce canal.
