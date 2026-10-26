; ReCoF Launcher installer.
;
; Built by `cargo xtask pack-installer`, which passes VERSION,
; VERSION_NUMERIC, EXE, LICENSE and OUTFILE on the command line.
;
; A per user install into %LOCALAPPDATA%\Programs: no administrator rights,
; no UAC prompt, and the launcher can replace its own executable when it
; updates itself, which it could not do inside Program Files.
;
; It installs the launcher and nothing else. The uninstaller removes the
; launcher, its shortcuts and its entry in the list of installed apps. The
; launcher's own data in %LOCALAPPDATA%\ReCoF Launcher goes only when the
; user ticks that box, and even then the save backups stay. A ReCoF copy
; made with the launcher is never touched here: the launcher removes that
; itself, from Maintenance.

Unicode true
ManifestDPIAware true
SetCompressor /SOLID lzma
RequestExecutionLevel user

!ifndef VERSION | VERSION_NUMERIC | EXE | LICENSE | OUTFILE
  !error "build this with cargo xtask pack-installer"
!endif

!define APP_NAME "ReCoF Launcher"
!define APP_EXE "ReCoF-Launcher.exe"
!define APP_KEY "Software\ReCoF Launcher"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\ReCoF Launcher"
!define DATA_DIR "$LOCALAPPDATA\ReCoF Launcher"

Name "${APP_NAME}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\ReCoF Launcher"
InstallDirRegKey HKCU "${APP_KEY}" "InstallDir"
BrandingText "${APP_NAME} ${VERSION}"

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"

!define MUI_ABORTWARNING
!define MUI_LANGDLL_REGISTRY_ROOT HKCU
!define MUI_LANGDLL_REGISTRY_KEY "${APP_KEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "InstallerLanguage"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${LICENSE}"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"
!insertmacro MUI_PAGE_FINISH

!define MUI_UNCONFIRMPAGE_TEXT_TOP "$(UnConfirmNote)"
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_COMPONENTS
!insertmacro MUI_UNPAGE_INSTFILES

; The launcher's own languages, English first as the fallback.
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "German"
!insertmacro MUI_LANGUAGE "French"
!insertmacro MUI_LANGUAGE "Italian"
!insertmacro MUI_LANGUAGE "Polish"
!insertmacro MUI_LANGUAGE "PortugueseBR"
!insertmacro MUI_LANGUAGE "Russian"
!insertmacro MUI_LANGUAGE "Turkish"
!insertmacro MUI_LANGUAGE "Ukrainian"
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_RESERVEFILE_LANGDLL

VIProductVersion "${VERSION_NUMERIC}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductName" "${APP_NAME}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductVersion" "${VERSION}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileVersion" "${VERSION}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileDescription" "${APP_NAME} ${VERSION} setup"
VIAddVersionKey /LANG=${LANG_ENGLISH} "LegalCopyright" "MIT License"

; English
LangString SecDesktop ${LANG_ENGLISH} "Desktop shortcut"
LangString DescLauncher ${LANG_ENGLISH} "The launcher, with an entry in the Start menu. It needs no administrator rights and keeps itself up to date."
LangString DescDesktop ${LANG_ENGLISH} "A shortcut to the launcher on the desktop."
LangString UnSecData ${LANG_ENGLISH} "Settings, logs and downloaded releases"
LangString UnDescLauncher ${LANG_ENGLISH} "The launcher, its shortcuts and its entry in the list of installed apps."
LangString UnDescData ${LANG_ENGLISH} "What the launcher keeps in %LOCALAPPDATA%\ReCoF Launcher, except the save backups, which always stay."
LangString UnConfirmNote ${LANG_ENGLISH} "This removes the launcher only. A ReCoF copy installed with it stays where it is: to remove it too, use Maintenance, Uninstall in the launcher first. Save backups always stay."

; Spanish
LangString SecDesktop ${LANG_SPANISH} "Acceso directo en el escritorio"
LangString DescLauncher ${LANG_SPANISH} "El launcher, con una entrada en el menú Inicio. No necesita permisos de administrador y se actualiza solo."
LangString DescDesktop ${LANG_SPANISH} "Un acceso directo al launcher en el escritorio."
LangString UnSecData ${LANG_SPANISH} "Ajustes, registros y versiones descargadas"
LangString UnDescLauncher ${LANG_SPANISH} "El launcher, sus accesos directos y su entrada en la lista de aplicaciones instaladas."
LangString UnDescData ${LANG_SPANISH} "Lo que el launcher guarda en %LOCALAPPDATA%\ReCoF Launcher, salvo las copias de seguridad de las partidas, que se quedan siempre."
LangString UnConfirmNote ${LANG_SPANISH} "Esto quita solo el launcher. Una copia de ReCoF instalada con él se queda donde está: para quitarla también, usa antes Mantenimiento, Desinstalar en el launcher. Las copias de seguridad de las partidas se quedan siempre."

; German (first draft)
LangString SecDesktop ${LANG_GERMAN} "Verknüpfung auf dem Desktop"
LangString DescLauncher ${LANG_GERMAN} "Der Launcher, mit einem Eintrag im Startmenü. Er braucht keine Administratorrechte und hält sich selbst aktuell."
LangString DescDesktop ${LANG_GERMAN} "Eine Verknüpfung zum Launcher auf dem Desktop."
LangString UnSecData ${LANG_GERMAN} "Einstellungen, Protokolle und heruntergeladene Versionen"
LangString UnDescLauncher ${LANG_GERMAN} "Der Launcher, seine Verknüpfungen und sein Eintrag in der Liste der installierten Apps."
LangString UnDescData ${LANG_GERMAN} "Was der Launcher in %LOCALAPPDATA%\ReCoF Launcher ablegt, außer den Sicherungen der Spielstände, die immer bleiben."
LangString UnConfirmNote ${LANG_GERMAN} "Dies entfernt nur den Launcher. Eine damit installierte ReCoF-Kopie bleibt, wo sie ist: um sie auch zu entfernen, nutze vorher Wartung, Deinstallieren im Launcher. Sicherungen der Spielstände bleiben immer erhalten."

; French (first draft)
LangString SecDesktop ${LANG_FRENCH} "Raccourci sur le bureau"
LangString DescLauncher ${LANG_FRENCH} "Le lanceur, avec une entrée dans le menu Démarrer. Il ne demande pas de droits d'administrateur et se met à jour tout seul."
LangString DescDesktop ${LANG_FRENCH} "Un raccourci vers le lanceur sur le bureau."
LangString UnSecData ${LANG_FRENCH} "Paramètres, journaux et versions téléchargées"
LangString UnDescLauncher ${LANG_FRENCH} "Le lanceur, ses raccourcis et son entrée dans la liste des applications installées."
LangString UnDescData ${LANG_FRENCH} "Ce que le lanceur garde dans %LOCALAPPDATA%\ReCoF Launcher, sauf les sauvegardes des parties, qui restent toujours."
LangString UnConfirmNote ${LANG_FRENCH} "Ceci supprime uniquement le lanceur. Une copie de ReCoF installée avec lui reste où elle est : pour la supprimer aussi, utilisez d'abord Maintenance, Désinstaller dans le lanceur. Les sauvegardes des parties restent toujours."

; Italian (first draft)
LangString SecDesktop ${LANG_ITALIAN} "Collegamento sul desktop"
LangString DescLauncher ${LANG_ITALIAN} "Il launcher, con una voce nel menu Start. Non servono diritti di amministratore e si aggiorna da solo."
LangString DescDesktop ${LANG_ITALIAN} "Un collegamento al launcher sul desktop."
LangString UnSecData ${LANG_ITALIAN} "Impostazioni, registri e versioni scaricate"
LangString UnDescLauncher ${LANG_ITALIAN} "Il launcher, i suoi collegamenti e la sua voce nell'elenco delle app installate."
LangString UnDescData ${LANG_ITALIAN} "Ciò che il launcher tiene in %LOCALAPPDATA%\ReCoF Launcher, tranne i backup dei salvataggi, che restano sempre."
LangString UnConfirmNote ${LANG_ITALIAN} "Questo rimuove solo il launcher. Una copia di ReCoF installata con esso resta dov'è: per rimuovere anche quella, usa prima Manutenzione, Disinstalla nel launcher. I backup dei salvataggi restano sempre."

; Polish (first draft)
LangString SecDesktop ${LANG_POLISH} "Skrót na pulpicie"
LangString DescLauncher ${LANG_POLISH} "Launcher z wpisem w menu Start. Nie wymaga uprawnień administratora i sam się aktualizuje."
LangString DescDesktop ${LANG_POLISH} "Skrót do launchera na pulpicie."
LangString UnSecData ${LANG_POLISH} "Ustawienia, dzienniki i pobrane wydania"
LangString UnDescLauncher ${LANG_POLISH} "Launcher, jego skróty i jego wpis na liście zainstalowanych aplikacji."
LangString UnDescData ${LANG_POLISH} "To, co launcher trzyma w %LOCALAPPDATA%\ReCoF Launcher, poza kopiami zapasowymi zapisów, które zostają zawsze."
LangString UnConfirmNote ${LANG_POLISH} "To usuwa tylko launcher. Kopia ReCoF zainstalowana za jego pomocą zostaje na miejscu: aby ją też usunąć, najpierw użyj w launcherze opcji Konserwacja, Odinstaluj. Kopie zapasowe zapisów zostają zawsze."

; Brazilian Portuguese (first draft)
LangString SecDesktop ${LANG_PORTUGUESEBR} "Atalho na área de trabalho"
LangString DescLauncher ${LANG_PORTUGUESEBR} "O launcher, com uma entrada no menu Iniciar. Não precisa de permissões de administrador e se atualiza sozinho."
LangString DescDesktop ${LANG_PORTUGUESEBR} "Um atalho para o launcher na área de trabalho."
LangString UnSecData ${LANG_PORTUGUESEBR} "Configurações, registros e versões baixadas"
LangString UnDescLauncher ${LANG_PORTUGUESEBR} "O launcher, seus atalhos e sua entrada na lista de aplicativos instalados."
LangString UnDescData ${LANG_PORTUGUESEBR} "O que o launcher guarda em %LOCALAPPDATA%\ReCoF Launcher, exceto os backups dos saves, que ficam sempre."
LangString UnConfirmNote ${LANG_PORTUGUESEBR} "Isto remove apenas o launcher. Uma cópia do ReCoF instalada com ele fica onde está: para removê-la também, use antes Manutenção, Desinstalar no launcher. Os backups dos saves ficam sempre."

; Russian (first draft)
LangString SecDesktop ${LANG_RUSSIAN} "Ярлык на рабочем столе"
LangString DescLauncher ${LANG_RUSSIAN} "Лаунчер с пунктом в меню Пуск. Ему не нужны права администратора, и он обновляется сам."
LangString DescDesktop ${LANG_RUSSIAN} "Ярлык лаунчера на рабочем столе."
LangString UnSecData ${LANG_RUSSIAN} "Настройки, журналы и загруженные выпуски"
LangString UnDescLauncher ${LANG_RUSSIAN} "Лаунчер, его ярлыки и его запись в списке установленных приложений."
LangString UnDescData ${LANG_RUSSIAN} "То, что лаунчер хранит в %LOCALAPPDATA%\ReCoF Launcher, кроме резервных копий сохранений, которые остаются всегда."
LangString UnConfirmNote ${LANG_RUSSIAN} "Будет удалён только лаунчер. Копия ReCoF, установленная с его помощью, останется на месте: чтобы удалить и её, сначала выберите в лаунчере Обслуживание, Удалить. Резервные копии сохранений остаются всегда."

; Turkish (first draft)
LangString SecDesktop ${LANG_TURKISH} "Masaüstü kısayolu"
LangString DescLauncher ${LANG_TURKISH} "Başlat menüsünde bir girişiyle başlatıcı. Yönetici izni gerektirmez ve kendini günceller."
LangString DescDesktop ${LANG_TURKISH} "Masaüstünde başlatıcıya bir kısayol."
LangString UnSecData ${LANG_TURKISH} "Ayarlar, günlükler ve indirilen sürümler"
LangString UnDescLauncher ${LANG_TURKISH} "Başlatıcı, kısayolları ve yüklü uygulamalar listesindeki girişi."
LangString UnDescData ${LANG_TURKISH} "Başlatıcının %LOCALAPPDATA%\ReCoF Launcher içinde tuttukları; her zaman kalan kayıt yedekleri hariç."
LangString UnConfirmNote ${LANG_TURKISH} "Bu yalnızca başlatıcıyı kaldırır. Onunla kurulan ReCoF kopyası olduğu yerde kalır: onu da kaldırmak için önce başlatıcıda Bakım, Kaldır seçeneğini kullanın. Kayıt yedekleri her zaman kalır."

; Ukrainian (first draft)
LangString SecDesktop ${LANG_UKRAINIAN} "Ярлик на стільниці"
LangString DescLauncher ${LANG_UKRAINIAN} "Лаунчер із пунктом у меню Пуск. Йому не потрібні права адміністратора, і він оновлюється сам."
LangString DescDesktop ${LANG_UKRAINIAN} "Ярлик лаунчера на стільниці."
LangString UnSecData ${LANG_UKRAINIAN} "Налаштування, журнали та завантажені випуски"
LangString UnDescLauncher ${LANG_UKRAINIAN} "Лаунчер, його ярлики та його запис у списку встановлених застосунків."
LangString UnDescData ${LANG_UKRAINIAN} "Те, що лаунчер зберігає в %LOCALAPPDATA%\ReCoF Launcher, крім резервних копій збережень, які лишаються завжди."
LangString UnConfirmNote ${LANG_UKRAINIAN} "Буде видалено лише лаунчер. Копія ReCoF, встановлена з його допомогою, лишиться на місці: щоб видалити і її, спершу виберіть у лаунчері Обслуговування, Видалити. Резервні копії збережень лишаються завжди."

; Simplified Chinese (first draft)
LangString SecDesktop ${LANG_SIMPCHINESE} "桌面快捷方式"
LangString DescLauncher ${LANG_SIMPCHINESE} "启动器本体，并在开始菜单中添加入口。无需管理员权限，并会自行更新。"
LangString DescDesktop ${LANG_SIMPCHINESE} "在桌面上创建启动器的快捷方式。"
LangString UnSecData ${LANG_SIMPCHINESE} "设置、日志和已下载的版本"
LangString UnDescLauncher ${LANG_SIMPCHINESE} "启动器、它的快捷方式，以及它在已安装应用列表中的条目。"
LangString UnDescData ${LANG_SIMPCHINESE} "启动器保存在 %LOCALAPPDATA%\ReCoF Launcher 中的内容，存档备份除外，存档备份始终保留。"
LangString UnConfirmNote ${LANG_SIMPCHINESE} "这只会移除启动器。用它安装的 ReCoF 副本会留在原处：如需一并移除，请先在启动器中使用 维护，卸载。存档备份始终保留。"

Function .onInit
  ${IfNot} ${Silent}
    !insertmacro MUI_LANGDLL_DISPLAY
  ${EndIf}
FunctionEnd

Function un.onInit
  !insertmacro MUI_UNGETLANGUAGE
FunctionEnd

Section "!${APP_NAME}" SecLauncher
  SectionIn RO
  SetOutPath "$INSTDIR"
  File "/oname=${APP_EXE}" "${EXE}"
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateShortcut "$SMPROGRAMS\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"

  WriteRegStr HKCU "${APP_KEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\${APP_EXE}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "ReCoF"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "URLInfoAbout" "https://github.com/caixax/ReCoF-Launcher"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
  ; No DisplayVersion: the launcher updates itself in place, so a version
  ; written here would be wrong after the first update.
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "EstimatedSize" "$0"
SectionEnd

Section "$(SecDesktop)" SecDesktop
  CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
SectionEnd

Section "un.${APP_NAME}" UnSecLauncher
  SectionIn RO
  Delete "$INSTDIR\${APP_EXE}"
  ; What self-replace may leave behind when the launcher updated itself.
  Delete "$INSTDIR\.ReCoF-Launcher.*"
  Delete "$INSTDIR\Uninstall.exe"
  ; Not /r: the folder may be one the user picked and holds other things.
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\${APP_NAME}.lnk"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  DeleteRegKey HKCU "${UNINSTALL_KEY}"
  DeleteRegKey HKCU "${APP_KEY}"
SectionEnd

Section /o "un.$(UnSecData)" UnSecData
  ; The paths the launcher writes, one by one, so the backups folder and
  ; anything unknown are left where they are.
  Delete "${DATA_DIR}\settings.toml"
  RMDir /r "${DATA_DIR}\logs"
  RMDir /r "${DATA_DIR}\cache"
  RMDir /r "${DATA_DIR}\versions"
  RMDir "${DATA_DIR}"
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecLauncher} "$(DescLauncher)"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "$(DescDesktop)"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

!insertmacro MUI_UNFUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${UnSecLauncher} "$(UnDescLauncher)"
  !insertmacro MUI_DESCRIPTION_TEXT ${UnSecData} "$(UnDescData)"
!insertmacro MUI_UNFUNCTION_DESCRIPTION_END
