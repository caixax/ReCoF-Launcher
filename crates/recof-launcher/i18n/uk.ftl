# ReCoF Launcher, українська.
#
# ПЕРШИЙ ВАРІАНТ: цей переклад ще не перевіряв носій мови.
# Виправлення вітаються: https://github.com/caixax/ReCoF-Launcher
#
# Стиль: прості речення, без довгого й середнього тире.

app-name = ReCoF Launcher
app-tagline = Встановлювач і запускач ReCoF

## Кнопки й слова, що трапляються всюди

button-next = Далі
button-back = Назад
button-cancel = Скасувати
button-close = Закрити
button-continue = Продовжити
button-ok = Гаразд
button-yes = Так
button-no = Ні
button-browse = Огляд...
button-retry = Спробувати знову
button-copy = Копіювати
button-open-folder = Відкрити теку
button-finish = Готово
button-skip = Пропустити
button-details = Подробиці
word-yes = так
word-no = ні
word-unknown = невідомо

## Перший запуск: мова

firstrun-language-title = Оберіть мову
firstrun-language-help = Це можна змінити згодом у налаштуваннях.
language-not-reviewed = Перший варіант, носій мови його ще не перевіряв
language-not-reviewed-notice = Цей переклад є першим варіантом і ще не перевірений носієм мови. Виправлення вітаються.

## Перший запуск: про що йдеться

intro-1-title = Що таке ReCoF
intro-1-body = ReCoF, це поліпшена перекомпіляція Cry of Fear. Ця програма її встановлює, оновлює та запускає. ReCoF і ця програма, це проєкти окремі від самої гри.
intro-2-title = Вашу копію зі Steam ніхто не чіпає
intro-2-body = Програма читає вашу інсталяцію Steam і копіює її до теки, яку ви оберете. Вона ніколи нічого не записує, не видаляє й не перейменовує всередині теки Steam, тож Cry of Fear працює так само, як зараз, і ви можете повернутися до неї будь-коли.
intro-3-title = Окрема копія і ваш поступ
intro-3-body = Оскільки ReCoF живе у власній копії, йому потрібно місце на всю гру, близько { $size }. Збереження, налаштування та відкритий вміст можна перенести з копії Steam під час встановлення або згодом через меню. У зворотному напрямку нічого не копіюється.

## Пошук гри

detect-title = Пошук Cry of Fear
detect-searching = Триває пошук вашої інсталяції Steam...
detect-found = Знайдено в { $path }
detect-not-found = Cry of Fear не знайдено
detect-not-found-help = Можливо, Steam не встановлено або гра лежить у теці, якої програма не бачить. Оберіть теку, де є cof.exe.
detect-pick-folder = Обрати теку гри
detect-pick-folder-help = Тека, де є cof.exe
detect-library = Бібліотека Steam: { $path }
detect-buildid-ok = Збірка { $buildid }, саме та, під яку зібрано ReCoF
detect-buildid-wrong = Це збірка { $found }. ReCoF зібрано під збірку { $expected }. Він може працювати неправильно, і проблеми цілком імовірні.
detect-buildid-unknown = Прочитати номер збірки Steam не вдалося. Для теки, обраної вручну, це нормально.
detect-size = Файлів: { $files }, { $size }

## Чим виявилися файли

detect-check-title = Перевірка файлів гри
detect-check-clean = Усі файли збігаються з роздрібною версією 1.6a.
detect-check-notices = Деякі файли не ті, що постачаються з роздрібною версією 1.6a. Встановленню це не заважає: копія ваша, і ReCoF усе одно замінює ці файли. Тут це перелічено, щоб ви знали, що копіюєте.
verdict-retail = збігається з роздрібною версією
verdict-patched = роздрібна версія із застосованим латанням на 4 ГБ
verdict-unknown = файл, незнайомий програмі, SHA-256 { $sha256 }
verdict-missing = відсутній
detect-unexpected-title = Зайві програми в теці гри
detect-unexpected-help = Роздрібна версія Cry of Fear їх не містить. Зазвичай вони походять з аматорського латання.
detect-mods-title = Відомі модифікації

variant-cof-enhanced = версія, яку ставить латання Cry of Fear Enhanced
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = заміна рушія на Xash3D
mod-xash-menu = меню Xash3D
mod-bunnymod-xt = Bunnymod XT

## Куди встановлювати

install-title = Встановити ReCoF
install-destination = Встановити в
install-destination-help = Нова тека, поза Steam. ReCoF потрібна повна копія гри.
install-size-line = Копії потрібно { $size }. На { $volume } вільно { $available }.
install-space-ok = Місця вистачає.
install-space-short = На { $volume } бракує місця. Не вистачає { $missing }.
install-space-explain = Крім самої копії, програма резервує місце під кілька останніх версій і під резервні копії збережень, щоб ви могли відкотитися і щоб оновлення ніколи не починалося без шляху назад.

dest-inside-steam = Ця тека лежить усередині теки Steam. ReCoF має жити назовні, щоб оновлення Steam ніколи не могло його перезаписати.
dest-inside-steamapps = Ця тека лежить у steamapps бібліотеки Steam, де файлами керує Steam. Оберіть інше місце.
dest-protected = Ця тека потребує прав адміністратора й не годиться для гри. Оберіть щось серед власних файлів.
dest-is-vanilla = Це копія гри зі Steam. ReCoF ставиться в окрему теку.
dest-drive-root = Це корінь диска. Оберіть теку всередині нього.
dest-not-empty = У цій теці вже лежить { $entries } сторонніх файлів. Оберіть порожню або нову теку.
dest-unusable = Цей шлях використати не можна.

## Встановлення

install-copying = Копіювання гри: { $current }
install-copying-counted = Копіювання гри, { $done } з { $total }
install-applying = Застосування ReCoF { $version }
install-finishing = Завершення
install-done = ReCoF { $version } встановлено.
install-cancel-confirm = Зупинити встановлення? Усе скопійоване буде видалено. Вашої копії зі Steam це не торкнеться.
install-cancelled = Встановлення скасовано, нічого недоробленого не залишилося.
install-offer-import = Перенести мої збереження, налаштування та відкритий вміст із копії Steam
install-offer-shortcuts = Створити ярлик на стільниці
install-release-title = Що буде встановлено
install-release-version = ReCoF { $version }
install-release-download = Завантажується з GitHub і перевіряється за підписом, перш ніж щось буде скопійовано.
install-release-stored = Уже завантажено й перевірено.
install-release-file = З файлу { $name }
install-release-file-help = Підписаний маніфест поруч із zip перевіряється, перш ніж щось буде скопійовано.
install-release-file-clear = Використати опубліковану версію
install-release-checking = Пошук останньої версії
install-release-offline = Не вдалося зв'язатися з GitHub. Спробуйте ще раз або встановіть із файлу.
install-release-try-beta = У стабільному каналі ще нічого не випущено. Тестувальники можуть перейти на бету.

## Головний екран

home-play = Грати в ReCoF
home-play-vanilla = Грати в оригінал
home-version-installed = Встановлено ReCoF { $version }
home-no-install = ReCoF ще не встановлено
home-install-now = Встановити ReCoF
home-channel = Канал: { $channel }
home-checking = Перевірка оновлень...
home-up-to-date = Усе актуальне
home-update-available = Доступний ReCoF { $version }
home-launcher-update-available = Доступна нова версія програми: { $version }
home-offline = Зв'язатися з GitHub не вдалося. Програма працює й без цього.

## Запуск гри

launch-steam-not-running = Steam не запущено
launch-steam-not-running-help = Cry of Fear користується Steam під час роботи. Спершу запустіть Steam.
launch-open-steam = Відкрити Steam
launch-start-anyway = Усе одно запустити
launch-starting = Запуск ReCoF...
launch-failed = Запустити ReCoF не вдалося.
launch-already-running = ReCoF уже запущено.

launch-options-title = Параметри запуску
launch-resolution = Роздільність
launch-resolution-default = Як вирішить гра
launch-windowed = У вікні
launch-widescreen = Широкий екран
launch-3dmenu = Тривимірне меню
launch-3dmenu-help = Роздрібна версія запускається з вимкненим тривимірним меню. Якщо увімкнути, використовується початкове анімоване меню.
launch-extra-args = Додаткові аргументи
launch-extra-args-help = Передаються до cof.exe як є. Залиште порожнім, якщо не знаєте, що вам потрібно.
launch-preview = Командний рядок

launch-cvars-title = Параметри ReCoF
launch-cvars-none = У цій версії ReCoF немає власних параметрів.
launch-cvars-from-release = Вони походять зі встановленої у вас версії ReCoF.
launch-cvars-defaults = Повернути типові значення

## Оновлення

update-title = Оновлення
update-available-title = Доступний ReCoF { $version }
update-installed-version = У вас { $version }
update-changelog = Що змінилося
update-no-changelog = Ця версія не має опису змін.
update-install = Оновити зараз
update-later = Пізніше
update-downloading = Завантаження { $name }
update-verifying = Перевірка підпису та контрольних сум
update-extracting = Розпакування
update-applying = Застосування оновлення
update-done = Оновлено до ReCoF { $version }.
update-backup-first = Спершу створюється резервна копія збережень
update-launcher-first = Цій версії ReCoF потрібна програма { $needed } або новіша. Спершу оновіть програму.
update-launcher-title = Оновлення програми
update-launcher-restart = Програма перезапуститься, щоб завершити оновлення.
update-from-file = Встановити з файлу
update-from-file-help = Оберіть zip-архів випуску. Підписаний маніфест має лежати поряд.
changelog-note = Примітка
changelog-tip = Порада
changelog-important = Важливо
changelog-warning = Попередження
changelog-caution = Обережно

channel-title = Канал випусків
channel-stable = Стабільний
channel-beta = Бета
channel-beta-help = У каналі бета є попередні випуски. Чекайте на проблеми й бережіть резервні копії.

## Відкат

rollback-title = Повернутися до старішої версії
rollback-none = Жодної старішої версії не збережено.
rollback-to = Повернутися до { $version }
rollback-offer-backup = Відновити збереження, відкладені перед тим оновленням
rollback-done = ReCoF { $version } відновлено.

## Обслуговування

maintain-title = Обслуговування
maintain-verify = Перевірити та полагодити
maintain-verify-help = Рахує контрольну суму кожного файлу ReCoF за маніфестом і перезаписує все, що не збіглося.
maintain-backup = Резервна копія збережень
maintain-restore = Відновити збереження
maintain-import = Перенести поступ із копії Steam
maintain-move = Перемістити інсталяцію
maintain-uninstall = Видалити
maintain-shortcuts = Ярлики
maintain-logs = Журнали

verify-running = Перевірено { $done } з { $total }
verify-intact = Усі файли збігаються.
verify-broken = Не збігається файлів: { $count }.
verify-repair = Полагодити їх
verify-repaired = Перезаписано файлів: { $count }.
verify-cannot-repair = Версії, з якої ці файли, у сховищі більше немає. Щоб полагодити, перевстановіть або оновіться.

## Резервні копії збережень

backup-title = Резервні копії збережень
backup-what = Зберігаються: збереження гри, налаштування, відкритий вміст і ваші файли конфігурації.
backup-create = Створити копію зараз
backup-created = Збережено в { $name }
backup-none = Резервних копій поки немає.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = Відновити { $name }? Збереження у вашій копії ReCoF буде замінено.
backup-restored = Резервну копію відновлено.
backup-verify-failed = Резервну копію не вдалося прочитати назад після запису, тож довіряти їй не можна. Нічого не змінено.
backup-auto = Створена автоматично перед оновленням до { $version }

## Перенесення з копії Steam

import-title = Перенести поступ
import-explain = Копіює збереження, налаштування та відкритий вміст з інсталяції Steam до копії ReCoF. До інсталяції Steam нічого не записується, і у зворотному напрямку нічого не копіюється.
import-nothing-found = У копії Steam немає збережень для перенесення.
import-overwrite-title = Це замінить файли, які у вас уже є
import-overwrite-body = У копії ReCoF буде замінено файлів: { $count }. Перед цим створюється резервна копія.
import-done = Перенесено файлів: { $count }.

## Переміщення

move-title = Перемістити інсталяцію
move-to = Перемістити в
move-running = Переміщення
move-done = Інсталяція тепер у { $path }.
move-same-volume = На тому самому диску це відбувається миттєво.

## Видалення

uninstall-title = Видалити ReCoF
uninstall-explain = Це видаляє копію ReCoF і збережені версії. Вашої інсталяції Steam це не торкається.
uninstall-export-first = Спершу вивантажити мої збереження
uninstall-keep-backups = Залишити мої резервні копії збережень
uninstall-confirm = Видалити { $path }?
uninstall-done = ReCoF видалено.

## Ярлики

shortcut-desktop = Ярлик на стільниці
shortcut-start-menu = Ярлик у меню «Пуск»
shortcut-steam = Додати ReCoF до Steam як сторонню гру
shortcut-steam-help = Це єдине, що програма записує поза власними теками. Steam має бути закритий, і перед цим зберігається копія shortcuts.vdf.
shortcut-steam-needs-steam-closed = Спершу закрийте Steam. Виходячи, Steam перезаписує цей файл, що скасує зміну.
shortcut-steam-added = ReCoF додано до Steam. Перезапустіть Steam, щоб його побачити.
shortcut-created = Ярлик створено.

## Повідомлення про помилку

bug-title = Повідомити про помилку
bug-explain = Тут збираються версії, результат останньої перевірки, відомості про вашу систему та останні рядки журналу. Перегляньте все, приберіть те, чим не хочете ділитися, і програма відкриє у браузері заздалегідь заповнене повідомлення.
bug-never-sends = Нікуди нічого не надсилається. Програма лише відкриває ваш браузер.
bug-collecting = Збирання відомостей
bug-open-issue = Відкрити повідомлення у браузері
bug-copied = Скопійовано до буфера обміну.

## Налаштування

settings-title = Налаштування
settings-language = Мова
settings-check-updates = Перевіряти оновлення під час запуску програми
settings-discord = Discord Rich Presence
settings-discord-help = Показує у вашому профілі Discord, що ви граєте в ReCoF, поки гра відкрита. Типово вимкнено.
settings-keep-versions = Скільки версій зберігати для відкату
settings-open-logs = Відкрити теку журналів
settings-about = Про програму

## Відновлення після перерваної операції

recovery-rolled-back = Перервану операцію скасовано. Інсталяція в тому самому стані, що й до її початку.
recovery-rolled-forward = Перервану операцію завершено. Інсталяція на версії { $version }.

## Помилки

err-io = Не вдалося прочитати або записати файл: { $path }
err-not-supported = У цій системі це недоступно.
err-registry = Не вдалося прочитати ключ реєстру Windows { $key }.
err-platform-api = Виклик { $api } завершився помилкою.
err-no-game-exe = У { $path } немає cof.exe, отже це не тека Cry of Fear.
err-no-game-dir = У { $path } немає теки cryoffear, отже це не тека Cry of Fear.
err-journal = Завершити операцію встановлення не вдалося. Нічого не лишається наполовину: під час наступного запуску програма її завершить або скасує.
err-interrupted = Операцію зупинено.

err-no-trusted-keys = У цій збірці програми немає ключа підпису, тож вона не може перевірити жодного випуску. Нічого не встановлено.
err-bad-signature = Підпис цього випуску не сходиться. Нічого не встановлено. Завантажте наново, а якщо не перестане, повідомте про це.
err-bad-manifest = Цей випуск описує себе так, як програма не приймає. Нічого не встановлено.
err-launcher-too-old = Цьому випуску потрібна новіша програма. Спершу оновіть програму.
err-wrong-build = Цей випуск зібрано під іншу версію Cry of Fear.
err-hash-mismatch = Завантажений файл не такий, яким має бути. Нічого не встановлено.

err-network = Зв'язатися з GitHub не вдалося. Перевірте з'єднання й спробуйте знову.
err-github = GitHub відповів помилкою: { $detail }
err-rate-limited = GitHub обмежує запити з цього з'єднання. Спробуйте за кілька хвилин.
err-no-release = У цьому каналі ще нічого не випущено.
err-command-line-too-long = Командний рядок мав би { $length } символів, а гра приймає щонайбільше { $limit }. Скоротіть додаткові аргументи.
