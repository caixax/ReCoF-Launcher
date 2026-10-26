# ReCoF Launcher, русский.
#
# ПЕРВЫЙ ВАРИАНТ: этот перевод ещё не проверял носитель языка.
# Исправления приветствуются: https://github.com/caixax/ReCoF-Launcher
#
# Стиль: простые предложения, без длинного и среднего тире.

app-name = ReCoF Launcher
app-tagline = Установщик и запускатор ReCoF

## Кнопки и слова, которые встречаются везде

button-next = Далее
button-back = Назад
button-cancel = Отмена
button-close = Закрыть
button-continue = Продолжить
button-ok = ОК
button-yes = Да
button-no = Нет
button-browse = Обзор...
button-retry = Попробовать снова
button-copy = Копировать
button-open-folder = Открыть папку
button-finish = Готово
button-skip = Пропустить
button-details = Подробности
word-yes = да
word-no = нет
word-unknown = неизвестно

## Первый запуск: язык

firstrun-language-title = Выберите язык
firstrun-language-help = Это можно изменить позже в настройках.
language-not-reviewed = Первый вариант, носитель языка его ещё не проверял
language-not-reviewed-notice = Этот перевод является первым вариантом и ещё не проверен носителем языка. Исправления приветствуются.

## Первый запуск: о чём речь

intro-1-title = Что такое ReCoF
intro-1-body = ReCoF, это улучшенная перекомпиляция Cry of Fear. Эта программа устанавливает её, обновляет и запускает. ReCoF и эта программа, это проекты отдельные от самой игры.
intro-2-title = Вашу копию из Steam никто не трогает
intro-2-body = Программа читает вашу установку Steam и копирует её в папку, которую вы выберете. Она никогда ничего не записывает, не удаляет и не переименовывает внутри папки Steam, поэтому Cry of Fear работает точно так же, как сейчас, и вы можете вернуться к ней в любой момент.
intro-3-title = Отдельная копия и ваш прогресс
intro-3-body = Поскольку ReCoF живёт в собственной копии, ему нужно место под всю игру, около { $size }. Сохранения, настройки и открытое содержимое можно перенести из копии Steam во время установки или позже через меню. В обратную сторону ничего не копируется.

## Поиск игры

detect-title = Поиск Cry of Fear
detect-searching = Идёт поиск вашей установки Steam...
detect-found = Найдена в { $path }
detect-not-found = Cry of Fear не найдена
detect-not-found-help = Возможно, Steam не установлен или игра лежит в папке, которую программа не видит. Выберите папку, в которой находится cof.exe.
detect-pick-folder = Выбрать папку игры
detect-pick-folder-help = Папка, в которой находится cof.exe
detect-library = Библиотека Steam: { $path }
detect-buildid-ok = Сборка { $buildid }, именно та, под которую собран ReCoF
detect-buildid-wrong = Это сборка { $found }. ReCoF собран под сборку { $expected }. Он может работать неправильно, и проблемы вполне вероятны.
detect-buildid-unknown = Прочитать номер сборки Steam не удалось. Для папки, выбранной вручную, это нормально.
detect-size = Файлов: { $files }, { $size }

## Чем оказались файлы

detect-check-title = Проверка файлов игры
detect-check-clean = Все файлы совпадают с розничной версией 1.6a.
detect-check-notices = Некоторые файлы не те, что поставляются с розничной версией 1.6a. Установке это не мешает: копия ваша, и ReCoF всё равно заменяет эти файлы. Здесь это перечислено, чтобы вы знали, что копируете.
verdict-retail = совпадает с розничной версией
verdict-patched = розничная версия с применённым патчем на 4 ГБ
verdict-unknown = файл, незнакомый программе, SHA-256 { $sha256 }
verdict-missing = отсутствует
detect-unexpected-title = Лишние программы в папке игры
detect-unexpected-help = Розничная версия Cry of Fear их не содержит. Обычно они приходят из любительского патча.
detect-mods-title = Известные модификации

variant-cof-enhanced = версия, которую ставит патч Cry of Fear Enhanced
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = замена движка на Xash3D
mod-xash-menu = меню Xash3D
mod-bunnymod-xt = Bunnymod XT

## Куда устанавливать

install-title = Установить ReCoF
install-destination = Установить в
install-destination-help = Новая папка, вне Steam. ReCoF нужна полная копия игры.
install-size-line = Копии требуется { $size }. На { $volume } свободно { $available }.
install-space-ok = Места хватает.
install-space-short = На { $volume } недостаточно места. Не хватает { $missing }.
install-space-explain = Кроме самой копии программа резервирует место под несколько последних версий и под резервные копии сохранений, чтобы вы могли откатиться и чтобы обновление никогда не начиналось без пути назад.

dest-inside-steam = Эта папка находится внутри папки Steam. ReCoF должен жить снаружи, чтобы обновление Steam никогда не смогло его перезаписать.
dest-inside-steamapps = Эта папка находится в steamapps библиотеки Steam, где файлами управляет Steam. Выберите другое место.
dest-protected = Эта папка требует прав администратора и не подходит для игры. Выберите что-нибудь среди своих собственных файлов.
dest-is-vanilla = Это копия игры из Steam. ReCoF ставится в отдельную папку.
dest-drive-root = Это корень диска. Выберите папку внутри него.
dest-not-empty = В этой папке уже лежит { $entries } посторонних файлов. Выберите пустую или новую папку.
dest-unusable = Этот путь использовать нельзя.

## Установка

install-copying = Копирование игры: { $current }
install-copying-counted = Копирование игры, { $done } из { $total }
install-applying = Применение ReCoF { $version }
install-finishing = Завершение
install-done = ReCoF { $version } установлен.
install-cancel-confirm = Остановить установку? Всё скопированное будет удалено. Вашу копию из Steam это не затронет.
install-cancelled = Установка отменена, ничего недоделанного не осталось.
install-offer-import = Перенести мои сохранения, настройки и открытое содержимое из копии Steam
install-offer-shortcuts = Создать ярлык на рабочем столе
install-release-title = Что будет установлено
install-release-version = ReCoF { $version }
install-release-download = Загружается с GitHub и проверяется по подписи до того, как что-либо будет скопировано.
install-release-stored = Уже загружено и проверено.
install-release-file = Из { $name }
install-release-file-help = Подписанный манифест рядом с zip проверяется до того, как что-либо будет скопировано.
install-release-file-clear = Использовать опубликованную версию
install-release-checking = Поиск последней версии
install-release-offline = Не удалось связаться с GitHub. Попробуйте ещё раз или установите из файла.
install-release-try-beta = В стабильном канале пока ничего не выпущено. Тестировщики могут переключиться на бету.

## Главный экран

home-play = Играть в ReCoF
home-play-vanilla = Играть в оригинал
home-version-installed = Установлен ReCoF { $version }
home-no-install = ReCoF ещё не установлен
home-install-now = Установить ReCoF
home-channel = Канал: { $channel }
home-checking = Проверка обновлений...
home-up-to-date = Всё актуально
home-update-available = Доступен ReCoF { $version }
home-launcher-update-available = Доступна новая версия программы: { $version }
home-offline = Связаться с GitHub не удалось. Программа работает и без этого.

## Запуск игры

launch-steam-not-running = Steam не запущен
launch-steam-not-running-help = Cry of Fear использует Steam во время работы. Сначала запустите Steam.
launch-open-steam = Открыть Steam
launch-start-anyway = Всё равно запустить
launch-starting = Запуск ReCoF...
launch-failed = Запустить ReCoF не удалось.
launch-already-running = ReCoF уже запущен.

launch-options-title = Параметры запуска
launch-resolution = Разрешение
launch-resolution-default = Как решит игра
launch-windowed = В окне
launch-widescreen = Широкий экран
launch-3dmenu = Трёхмерное меню
launch-3dmenu-help = Розничная версия запускается с выключенным трёхмерным меню. Если включить, используется исходное анимированное меню.
launch-extra-args = Дополнительные аргументы
launch-extra-args-help = Передаются в cof.exe как есть. Оставьте пустым, если не знаете, что вам нужно.
launch-preview = Командная строка

launch-cvars-title = Параметры ReCoF
launch-cvars-none = В этой версии ReCoF нет собственных параметров.
launch-cvars-from-release = Они приходят из установленной у вас версии ReCoF.
launch-cvars-defaults = Вернуть значения по умолчанию

## Обновления

update-title = Обновление
update-available-title = Доступен ReCoF { $version }
update-installed-version = У вас { $version }
update-changelog = Что изменилось
update-no-changelog = У этой версии нет описания изменений.
update-install = Обновить сейчас
update-later = Позже
update-downloading = Загрузка { $name }
update-verifying = Проверка подписи и контрольных сумм
update-extracting = Распаковка
update-applying = Применение обновления
update-done = Обновлено до ReCoF { $version }.
update-backup-first = Сначала создаётся резервная копия сохранений
update-launcher-first = Этой версии ReCoF нужна программа { $needed } или новее. Сначала обновите программу.
update-launcher-title = Обновление программы
update-launcher-restart = Программа перезапустится, чтобы завершить обновление.
update-from-file = Установить из файла
update-from-file-help = Выберите zip-архив выпуска. Подписанный манифест должен лежать рядом.
changelog-note = Примечание
changelog-tip = Совет
changelog-important = Важно
changelog-warning = Предупреждение
changelog-caution = Осторожно

channel-title = Канал выпусков
channel-stable = Стабильный
channel-beta = Бета
channel-beta-help = В канале бета есть предварительные выпуски. Ждите проблем и храните резервные копии.

## Откат

rollback-title = Вернуться к более старой версии
rollback-none = Ни одной старой версии не сохранено.
rollback-to = Вернуться к { $version }
rollback-offer-backup = Восстановить сохранения, отложенные перед тем обновлением
rollback-done = ReCoF { $version } восстановлен.

## Обслуживание

maintain-title = Обслуживание
maintain-verify = Проверить и починить
maintain-verify-help = Считает контрольную сумму каждого файла ReCoF по манифесту и перезаписывает всё, что не совпало.
maintain-backup = Резервная копия сохранений
maintain-restore = Восстановить сохранения
maintain-import = Перенести прогресс из копии Steam
maintain-move = Переместить установку
maintain-uninstall = Удалить
maintain-shortcuts = Ярлыки
maintain-logs = Журналы

verify-running = Проверено { $done } из { $total }
verify-intact = Все файлы совпадают.
verify-broken = Не совпадает файлов: { $count }.
verify-repair = Починить их
verify-repaired = Перезаписано файлов: { $count }.
verify-cannot-repair = Версии, из которой эти файлы, в хранилище больше нет. Чтобы починить, переустановите или обновитесь.

## Резервные копии сохранений

backup-title = Резервные копии сохранений
backup-what = Сохраняются: сохранения игры, настройки, открытое содержимое и ваши файлы конфигурации.
backup-create = Создать копию сейчас
backup-created = Сохранено в { $name }
backup-none = Резервных копий пока нет.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = Восстановить { $name }? Сохранения в вашей копии ReCoF будут заменены.
backup-restored = Резервная копия восстановлена.
backup-verify-failed = Резервную копию не удалось прочитать обратно после записи, поэтому доверять ей нельзя. Ничего не изменено.
backup-auto = Создана автоматически перед обновлением до { $version }

## Перенос из копии Steam

import-title = Перенести прогресс
import-explain = Копирует сохранения, настройки и открытое содержимое из установки Steam в копию ReCoF. В установку Steam ничего не записывается, и в обратную сторону ничего не копируется.
import-nothing-found = В копии Steam нет сохранений для переноса.
import-overwrite-title = Это заменит файлы, которые у вас уже есть
import-overwrite-body = В копии ReCoF будет заменено файлов: { $count }. Перед этим создаётся резервная копия.
import-done = Перенесено файлов: { $count }.

## Перемещение

move-title = Переместить установку
move-to = Переместить в
move-running = Перемещение
move-done = Установка теперь находится в { $path }.
move-same-volume = На том же диске это происходит мгновенно.

## Удаление

uninstall-title = Удалить ReCoF
uninstall-explain = Это удаляет копию ReCoF и сохранённые версии. Вашу установку Steam это не трогает.
uninstall-export-first = Сначала выгрузить мои сохранения
uninstall-keep-backups = Оставить мои резервные копии сохранений
uninstall-confirm = Удалить { $path }?
uninstall-done = ReCoF удалён.

## Ярлыки

shortcut-desktop = Ярлык на рабочем столе
shortcut-start-menu = Ярлык в меню «Пуск»
shortcut-steam = Добавить ReCoF в Steam как стороннюю игру
shortcut-steam-help = Это единственное, что программа записывает вне собственных папок. Steam должен быть закрыт, и перед этим сохраняется копия shortcuts.vdf.
shortcut-steam-needs-steam-closed = Сначала закройте Steam. При выходе Steam перезаписывает этот файл, что отменит изменение.
shortcut-steam-added = ReCoF добавлен в Steam. Перезапустите Steam, чтобы его увидеть.
shortcut-created = Ярлык создан.

## Сообщение об ошибке

bug-title = Сообщить об ошибке
bug-explain = Здесь собираются версии, результат последней проверки, сведения о вашей системе и последние строки журнала. Просмотрите всё, уберите то, чем не хотите делиться, и программа откроет в браузере заранее заполненное сообщение.
bug-never-sends = Никуда ничего не отправляется. Программа только открывает ваш браузер.
bug-collecting = Сбор сведений
bug-open-issue = Открыть сообщение в браузере
bug-copied = Скопировано в буфер обмена.

## Настройки

settings-title = Настройки
settings-language = Язык
settings-check-updates = Проверять обновления при запуске программы
settings-discord = Discord Rich Presence
settings-discord-help = Показывает в вашем профиле Discord, что вы играете в ReCoF, пока игра открыта. По умолчанию выключено.
settings-keep-versions = Сколько версий хранить для отката
settings-open-logs = Открыть папку журналов
settings-about = О программе

## Восстановление после прерванной операции

recovery-rolled-back = Прерванная операция отменена. Установка в том же виде, что и до её начала.
recovery-rolled-forward = Прерванная операция завершена. Установка на версии { $version }.

## Ошибки

err-io = Не удалось прочитать или записать файл: { $path }
err-not-supported = В этой системе это недоступно.
err-registry = Не удалось прочитать ключ реестра Windows { $key }.
err-platform-api = Вызов { $api } завершился ошибкой.
err-no-game-exe = В { $path } нет cof.exe, значит это не папка Cry of Fear.
err-no-game-dir = В { $path } нет папки cryoffear, значит это не папка Cry of Fear.
err-journal = Завершить операцию установки не удалось. Ничего не остаётся наполовину: при следующем запуске программа её завершит или отменит.
err-interrupted = Операция остановлена.

err-no-trusted-keys = В этой сборке программы нет ключа подписи, поэтому она не может проверить ни один выпуск. Ничего не установлено.
err-bad-signature = Подпись этого выпуска не сходится. Ничего не установлено. Скачайте заново, а если не перестанет, сообщите об этом.
err-bad-manifest = Этот выпуск описывает себя так, как программа не принимает. Ничего не установлено.
err-launcher-too-old = Этому выпуску нужна более новая программа. Сначала обновите программу.
err-wrong-build = Этот выпуск собран под другую версию Cry of Fear.
err-hash-mismatch = Скачанный файл не такой, каким должен быть. Ничего не установлено.

err-network = Связаться с GitHub не удалось. Проверьте соединение и попробуйте снова.
err-github = GitHub ответил ошибкой: { $detail }
err-rate-limited = GitHub ограничивает запросы с этого соединения. Попробуйте через несколько минут.
err-no-release = В этом канале ещё ничего не выпущено.
err-command-line-too-long = Командная строка была бы длиной { $length } символов, а игра принимает не больше { $limit }. Сократите дополнительные аргументы.
