# ReCoF Launcher, español.
#
# Estilo: frases llanas, sin guion largo (U+2014) ni guion medio
# (U+2013) en ningún sitio. Usa comas, dos puntos, paréntesis o un guion
# normal.

app-name = ReCoF Launcher
app-tagline = Instalador y lanzador de ReCoF

## Botones y palabras que salen por todas partes

button-next = Siguiente
button-back = Atrás
button-cancel = Cancelar
button-close = Cerrar
button-continue = Continuar
button-ok = Aceptar
button-yes = Sí
button-no = No
button-browse = Examinar...
button-retry = Reintentar
button-copy = Copiar
button-open-folder = Abrir carpeta
button-finish = Terminar
button-skip = Saltar
button-details = Detalles
word-yes = sí
word-no = no
word-unknown = desconocido

## Primer arranque: idioma

firstrun-language-title = Elige tu idioma
firstrun-language-help = Puedes cambiarlo después en los ajustes.
language-not-reviewed = Primer borrador, pendiente de revisión por un hablante nativo
language-not-reviewed-notice = Esta traducción es un primer borrador y todavía no la ha revisado un hablante nativo. Se agradecen las correcciones.

## Primer arranque: qué es esto

intro-1-title = Qué es ReCoF
intro-1-body = ReCoF es una recompilación mejorada de Cry of Fear. Este lanzador lo instala, lo mantiene actualizado y lo arranca. ReCoF y este lanzador son proyectos aparte del juego original.
intro-2-title = Tu copia de Steam no se toca
intro-2-body = El lanzador lee tu instalación de Steam y la copia a una carpeta que elijas tú. Nunca escribe, borra ni renombra nada dentro de la carpeta de Steam, así que Cry of Fear sigue funcionando exactamente igual que ahora y puedes volver a él cuando quieras.
intro-3-title = Una copia aparte, y tu progreso
intro-3-body = Como ReCoF vive en su propia copia, necesita sitio para el juego entero, unos { $size }. Tus partidas, ajustes y desbloqueables se pueden traer desde la copia de Steam durante la instalación, o después desde el menú. Nunca se copia nada en sentido contrario.

## Encontrar el juego

detect-title = Buscando Cry of Fear
detect-searching = Buscando tu instalación de Steam...
detect-found = Encontrado en { $path }
detect-not-found = No se ha encontrado Cry of Fear
detect-not-found-help = Puede que Steam no esté instalado, o que el juego esté en una carpeta que el lanzador no ve. Elige la carpeta que contiene cof.exe.
detect-pick-folder = Elegir la carpeta del juego
detect-pick-folder-help = La carpeta que contiene cof.exe
detect-library = Biblioteca de Steam: { $path }
detect-buildid-ok = Build { $buildid }, la que ReCoF espera
detect-buildid-wrong = Esta es la build { $found }. ReCoF está hecho para la build { $expected }. Puede que no funcione bien, y es de esperar que dé problemas.
detect-buildid-unknown = No se ha podido leer la build de Steam. Es lo normal si has elegido la carpeta a mano.
detect-size = { $files } archivos, { $size }

## Qué han resultado ser los archivos

detect-check-title = Comprobando los archivos del juego
detect-check-clean = Todos los archivos coinciden con la versión retail 1.6a.
detect-check-notices = Algunos archivos no son los que trae la versión retail 1.6a. Esto no impide instalar: la copia es tuya y ReCoF sustituye esos archivos de todos modos. Se muestra aquí para que sepas qué estás copiando.
verdict-retail = coincide con retail
verdict-patched = retail, con el parche de 4 GB aplicado
verdict-unknown = no es un archivo que el lanzador conozca, SHA-256 { $sha256 }
verdict-missing = falta
detect-unexpected-title = Programas de más en la carpeta del juego
detect-unexpected-help = La versión retail de Cry of Fear no los incluye. Suelen venir de un parche de la comunidad.
detect-mods-title = Modificaciones conocidas

variant-cof-enhanced = la versión que instala el parche Cry of Fear Enhanced
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = sustitución del motor por Xash3D
mod-xash-menu = menú de Xash3D
mod-bunnymod-xt = Bunnymod XT

## Elegir dónde instalar

install-title = Instalar ReCoF
install-destination = Instalar en
install-destination-help = Una carpeta nueva, fuera de Steam. ReCoF necesita una copia completa del juego.
install-size-line = La copia necesita { $size }. En { $volume } quedan { $available } libres.
install-space-ok = Hay sitio.
install-space-short = No hay sitio suficiente en { $volume }. Faltan { $missing }.
install-space-explain = Además de la copia, el lanzador reserva sitio para las últimas versiones y para los backups de partidas, para que puedas volver atrás y para que una actualización nunca empiece sin salida.

dest-inside-steam = Esa carpeta está dentro de la carpeta de Steam. ReCoF tiene que vivir fuera, para que una actualización de Steam no pueda sobrescribirlo.
dest-inside-steamapps = Esa carpeta está dentro del steamapps de una biblioteca de Steam, donde Steam gestiona los archivos. Elige otro sitio.
dest-protected = Esa carpeta necesita permisos de administrador y no es buen sitio para un juego. Elige algo dentro de tus propios archivos.
dest-is-vanilla = Esa es la copia de Steam del juego. ReCoF va en una carpeta aparte.
dest-drive-root = Esa es la raíz de una unidad. Elige una carpeta dentro de ella.
dest-not-empty = Esa carpeta ya contiene { $entries } archivos de otra cosa. Elige una carpeta vacía o una nueva.
dest-unusable = Esa ruta no se puede usar.

## Instalando

install-copying = Copiando el juego: { $current }
install-copying-counted = Copiando el juego, { $done } de { $total }
install-applying = Aplicando ReCoF { $version }
install-finishing = Terminando
install-done = ReCoF { $version } está instalado.
install-cancel-confirm = ¿Parar la instalación? Se borrará todo lo copiado hasta ahora. Tu copia de Steam no se ve afectada.
install-cancelled = La instalación se canceló y no quedó nada a medias.
install-offer-import = Traer mis partidas, ajustes y desbloqueables desde la copia de Steam
install-offer-shortcuts = Crear un acceso directo en el escritorio
install-release-title = Qué se instala
install-release-version = ReCoF { $version }
install-release-download = Se descarga de GitHub y se comprueba su firma antes de copiar nada.
install-release-stored = Ya está descargado y comprobado.
install-release-file = Desde { $name }
install-release-file-help = Antes de copiar nada se comprueba el manifiesto firmado que está junto al zip.
install-release-file-clear = Usar una versión publicada
install-release-checking = Buscando la última versión
install-release-offline = No se pudo conectar con GitHub. Vuelve a intentarlo o instala desde un archivo.
install-release-try-beta = Todavía no hay nada publicado en el canal estable. Si quieres probar, cambia a beta.

## La pantalla principal

home-play = Jugar a ReCoF
home-play-vanilla = Jugar al original
home-version-installed = ReCoF { $version } instalado
home-no-install = ReCoF todavía no está instalado
home-install-now = Instalar ReCoF
home-channel = Canal: { $channel }
home-checking = Buscando actualizaciones...
home-up-to-date = Todo al día
home-update-available = Hay disponible ReCoF { $version }
home-launcher-update-available = Hay una versión nueva del lanzador: { $version }
home-offline = No se ha podido conectar con GitHub. El lanzador funciona igual sin conexión.

## Arrancar el juego

launch-steam-not-running = Steam no está en ejecución
launch-steam-not-running-help = Cry of Fear usa Steam mientras corre. Abre Steam antes.
launch-open-steam = Abrir Steam
launch-start-anyway = Arrancar igualmente
launch-starting = Arrancando ReCoF...
launch-failed = No se ha podido arrancar ReCoF.
launch-already-running = ReCoF ya está en ejecución.

launch-options-title = Opciones de arranque
launch-resolution = Resolución
launch-resolution-default = La que decida el juego
launch-windowed = En ventana
launch-widescreen = Panorámico
launch-3dmenu = Menú 3D
launch-3dmenu-help = La versión retail arranca con el menú 3D desactivado. Al activarlo se usa el menú animado original.
launch-extra-args = Argumentos extra
launch-extra-args-help = Se pasan a cof.exe tal cual. Déjalo vacío si no sabes qué necesitas.
launch-preview = Línea de comandos

launch-cvars-title = Opciones de ReCoF
launch-cvars-none = Esta versión de ReCoF no trae opciones propias.
launch-cvars-from-release = Vienen de la versión de ReCoF que tienes instalada.
launch-cvars-defaults = Volver a los valores por defecto

## Actualizaciones

update-title = Actualizar
update-available-title = Hay disponible ReCoF { $version }
update-installed-version = Tienes la { $version }
update-changelog = Qué ha cambiado
update-no-changelog = Esta versión no trae notas.
update-install = Actualizar ahora
update-later = Más tarde
update-downloading = Descargando { $name }
update-verifying = Comprobando la firma y los hashes
update-extracting = Descomprimiendo
update-applying = Aplicando la actualización
update-done = Actualizado a ReCoF { $version }.
update-backup-first = Haciendo primero un backup de tus partidas
update-launcher-first = Esta versión de ReCoF necesita el lanzador { $needed } o posterior. Actualiza primero el lanzador.
update-launcher-title = Actualización del lanzador
update-launcher-restart = El lanzador se reiniciará para terminar de actualizarse.
update-from-file = Instalar desde un archivo
update-from-file-help = Elige un zip de release. Su manifiesto firmado tiene que estar al lado.
changelog-note = Nota
changelog-tip = Consejo
changelog-important = Importante
changelog-warning = Advertencia
changelog-caution = Precaución

channel-title = Canal de versiones
channel-stable = Estable
channel-beta = Beta
channel-beta-help = Beta incluye prelanzamientos. Es de esperar que den problemas, así que guarda tus backups.

## Volver atrás

rollback-title = Volver a una versión anterior
rollback-none = No se conserva ninguna versión anterior.
rollback-to = Volver a { $version }
rollback-offer-backup = Restaurar las partidas guardadas antes de aquella actualización
rollback-done = Se ha restaurado ReCoF { $version }.

## Mantenimiento

maintain-title = Mantenimiento
maintain-verify = Verificar y reparar
maintain-verify-help = Hashea todos los archivos de ReCoF contra el manifiesto y vuelve a escribir lo que no coincida.
maintain-backup = Backup de partidas
maintain-restore = Restaurar partidas
maintain-import = Importar progreso de la copia de Steam
maintain-move = Mover la instalación
maintain-uninstall = Desinstalar
maintain-shortcuts = Accesos directos
maintain-logs = Logs

verify-running = Comprobando { $done } de { $total }
verify-intact = Todos los archivos coinciden.
verify-broken = Hay { $count } archivos que no coinciden.
verify-repair = Repararlos
verify-repaired = Se han vuelto a escribir { $count } archivos.
verify-cannot-repair = La versión de la que salieron esos archivos ya no está en el almacén. Reinstala o actualiza para repararla.

## Backups de partidas

backup-title = Backups de partidas
backup-what = Se guardan: partidas, ajustes, desbloqueables y tus archivos de configuración.
backup-create = Hacer backup ahora
backup-created = Guardado en { $name }
backup-none = Todavía no hay ningún backup.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = ¿Restaurar { $name }? Se sustituirán las partidas de tu copia de ReCoF.
backup-restored = El backup se ha restaurado.
backup-verify-failed = El backup no se ha podido releer después de escribirlo, así que no es de fiar. No se ha cambiado nada.
backup-auto = Hecho automáticamente antes de actualizar a { $version }

## Importar desde la copia de Steam

import-title = Importar progreso
import-explain = Copia tus partidas, ajustes y desbloqueables desde la instalación de Steam a la copia de ReCoF. No se escribe nada dentro de la instalación de Steam, y nunca se copia nada en sentido contrario.
import-nothing-found = La copia de Steam no tiene partidas que importar.
import-overwrite-title = Esto va a sustituir archivos que ya tienes
import-overwrite-body = Se sustituirán { $count } archivos de la copia de ReCoF. Antes se hace un backup.
import-done = Se han importado { $count } archivos.

## Mover

move-title = Mover la instalación
move-to = Mover a
move-running = Moviendo
move-done = La instalación está ahora en { $path }.
move-same-volume = En el mismo disco esto es inmediato.

## Desinstalar

uninstall-title = Desinstalar ReCoF
uninstall-explain = Esto borra la copia de ReCoF y las versiones guardadas. Tu instalación de Steam no se toca.
uninstall-export-first = Exportar primero mis partidas
uninstall-keep-backups = Conservar mis backups de partidas
uninstall-confirm = ¿Borrar { $path }?
uninstall-done = Se ha borrado ReCoF.

## Accesos directos

shortcut-desktop = Acceso directo en el escritorio
shortcut-start-menu = Acceso directo en el menú inicio
shortcut-steam = Añadir ReCoF a Steam como juego no-Steam
shortcut-steam-help = Es lo único que el lanzador escribe fuera de sus propias carpetas. Steam tiene que estar cerrado, y antes se guarda una copia de shortcuts.vdf.
shortcut-steam-needs-steam-closed = Cierra Steam primero. Steam reescribe ese archivo al salir, lo que desharía el cambio.
shortcut-steam-added = ReCoF se ha añadido a Steam. Reinicia Steam para verlo.
shortcut-created = Acceso directo creado.

## Reportar un bug

bug-title = Reportar un bug
bug-explain = Esto recoge las versiones, el resultado de la última verificación, los datos de tu sistema y las últimas líneas del log. Revísalo, quita lo que prefieras no compartir, y el lanzador abrirá en tu navegador una incidencia ya rellenada.
bug-never-sends = No se envía nada a ningún sitio. El lanzador solo abre tu navegador.
bug-collecting = Recogiendo datos
bug-open-issue = Abrir la incidencia en mi navegador
bug-copied = Copiado al portapapeles.

## Ajustes

settings-title = Ajustes
settings-language = Idioma
settings-check-updates = Buscar actualizaciones al abrir el lanzador
settings-discord = Discord Rich Presence
settings-discord-help = Muestra en tu perfil de Discord que estás jugando a ReCoF, mientras el juego está abierto. Desactivado por defecto.
settings-keep-versions = Versiones que se conservan para volver atrás
settings-open-logs = Abrir la carpeta de logs
settings-about = Acerca de

## Recuperación tras una operación interrumpida

recovery-rolled-back = Se ha deshecho una operación que quedó interrumpida. La instalación está como estaba antes de empezarla.
recovery-rolled-forward = Se ha terminado una operación que quedó interrumpida. La instalación está en la versión { $version }.

## Errores

err-io = No se ha podido leer o escribir un archivo: { $path }
err-not-supported = Esto no está disponible en este sistema.
err-registry = No se ha podido leer la clave del registro de Windows { $key }.
err-platform-api = Ha fallado { $api }.
err-no-game-exe = { $path } no contiene cof.exe, así que no es una carpeta de Cry of Fear.
err-no-game-dir = { $path } no contiene una carpeta cryoffear, así que no es una carpeta de Cry of Fear.
err-journal = No se ha podido completar una operación de instalación. No queda nada a medias: el lanzador la terminará o la deshará en el próximo arranque.
err-interrupted = La operación se ha detenido.

err-no-trusted-keys = Esta compilación del lanzador no tiene clave de firma, así que no puede verificar ninguna versión. No se ha instalado nada.
err-bad-signature = La firma de esta versión no cuadra. No se ha instalado nada. Vuelve a descargarla, y si sigue fallando, repórtalo.
err-bad-manifest = Esta versión se describe a sí misma de una forma que el lanzador no acepta. No se ha instalado nada.
err-launcher-too-old = Esta versión necesita un lanzador más nuevo. Actualiza primero el lanzador.
err-wrong-build = Esta versión está hecha para otra versión de Cry of Fear.
err-hash-mismatch = Un archivo descargado no es el que debería ser. No se ha instalado nada.

err-network = No se ha podido conectar con GitHub. Comprueba tu conexión e inténtalo otra vez.
err-github = GitHub ha respondido con un error: { $detail }
err-rate-limited = GitHub está limitando las peticiones de esta conexión. Inténtalo dentro de unos minutos.
err-no-release = Todavía no se ha publicado ninguna versión en este canal.
err-command-line-too-long = La línea de comandos tendría { $length } caracteres y el juego admite como mucho { $limit }. Acorta los argumentos adicionales.

## Ajustes, por pestañas

settings-tab-look = Aspecto
settings-tab-language = Idioma
settings-tab-updates = Actualizaciones
settings-tab-behaviour = Comportamiento
settings-tab-about = Acerca de

## Aspecto

look-title = Aspecto
look-show-hero = Usar la imagen de fondo
look-hero-dim = Cuánto se oscurece el fondo
look-hero-dim-help = Lo bastante para que el texto de encima se siga leyendo.
look-grain = Grano de película
look-command-line = Mostrar la línea de comandos en la pantalla principal

look-artwork-title = Tu propio arte
look-artwork-help = El lanzador no incluye ninguna imagen del juego. Pon las tuyas en esta carpeta y las usará: hero.png para el fondo, logo.png para el título, icon.png para el icono de la ventana. Se quedan en esta máquina y nunca entran en un release.
look-open-theme = Abrir la carpeta de arte
look-reload-art = Recargar el arte
look-using-art = Usando tu arte: { $pieces }
look-no-art = No se ha encontrado arte, así que el lanzador dibuja el suyo.

## Comportamiento

behaviour-minimize = Minimizar el lanzador al arrancar el juego
behaviour-close = Cerrar el lanzador al arrancar el juego
behaviour-close-help = Tiene prioridad sobre minimizar.

## Acerca de

about-what = ReCoF Launcher instala, actualiza y arranca ReCoF, la recompilación mejorada de Cry of Fear. Copia tu instalación de Steam a una carpeta que elijas tú y nunca escribe dentro de la de Steam.
about-made-by = Hecho por { $author }, que también hace ReCoF.
about-version = Lanzador { $version }
about-licence = Licencia MIT. El código está en { $url }.
about-recof = ReCoF está en { $url }.
about-font = El título está compuesto en Archivo Black, con licencia SIL Open Font License 1.1.
about-not-affiliated = Sin relación con Team Psykskallar. Cry of Fear pertenece a sus autores; este lanzador solo instala una recompilación hecha a partir de tu propia copia.

## Créditos del arte

look-credits-title = Créditos del arte
look-credits-none = Este arte no vino con créditos.
look-credits-help = El arte no forma parte del lanzador. Viene del release de ReCoF y es de quien lo hiciera.
look-forget-art = Quitar el arte
