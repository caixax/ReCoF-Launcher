# ReCoF Launcher, 简体中文。
#
# 初稿：此翻译尚未经过母语者校对。欢迎提交修正：
# https://github.com/caixax/ReCoF-Launcher
#
# 风格：句子简明，任何地方都不使用长破折号或半长破折号。

app-name = ReCoF Launcher
app-tagline = ReCoF 的安装与启动程序

## 到处都会出现的按钮和词语

button-next = 下一步
button-back = 上一步
button-cancel = 取消
button-close = 关闭
button-continue = 继续
button-ok = 确定
button-yes = 是
button-no = 否
button-browse = 浏览...
button-retry = 重试
button-copy = 复制
button-open-folder = 打开文件夹
button-finish = 完成
button-skip = 跳过
button-details = 详情
word-yes = 是
word-no = 否
word-unknown = 未知

## 首次启动：语言

firstrun-language-title = 选择语言
firstrun-language-help = 之后可以在设置中更改。
language-not-reviewed = 初稿，尚未经过母语者校对
language-not-reviewed-notice = 此翻译是初稿，尚未经过母语者校对。欢迎提交修正。

## 首次启动：这是什么

intro-1-title = ReCoF 是什么
intro-1-body = ReCoF 是 Cry of Fear 的改进版重新编译。本程序负责安装、更新并启动它。ReCoF 和本程序都是独立于游戏本体的项目。
intro-2-title = 你的 Steam 副本不会被改动
intro-2-body = 本程序读取你的 Steam 安装，并复制到你选定的文件夹。它绝不会在 Steam 文件夹内写入、删除或重命名任何内容，因此 Cry of Fear 仍和现在完全一样，你随时可以回去玩。
intro-3-title = 一份独立副本，以及你的进度
intro-3-body = 由于 ReCoF 运行在自己的副本里，它需要整个游戏大小的空间，约 { $size }。你的存档、设置和解锁内容可以在安装时从 Steam 副本带过来，也可以之后从菜单导入。反方向绝不会复制任何东西。

## 查找游戏

detect-title = 正在查找 Cry of Fear
detect-searching = 正在查找你的 Steam 安装...
detect-found = 在 { $path } 找到
detect-not-found = 没有找到 Cry of Fear
detect-not-found-help = 可能没有安装 Steam，或者游戏在本程序看不到的文件夹里。请选择包含 cof.exe 的文件夹。
detect-pick-folder = 选择游戏文件夹
detect-pick-folder-help = 包含 cof.exe 的文件夹
detect-library = Steam 库：{ $path }
detect-buildid-ok = 版本号 { $buildid }，正是 ReCoF 所针对的版本
detect-buildid-wrong = 这是版本号 { $found }。ReCoF 针对的是版本号 { $expected }。它可能无法正常运行，出现问题是预料之中的。
detect-buildid-unknown = 无法读取 Steam 版本号。手动选择文件夹时这是正常的。
detect-size = { $files } 个文件，{ $size }

## 文件的检查结果

detect-check-title = 正在检查游戏文件
detect-check-clean = 所有文件都与零售版 1.6a 一致。
detect-check-notices = 有些文件不是零售版 1.6a 附带的。这不会阻止安装：副本是你的，而且 ReCoF 本来就会替换这些文件。这里列出来只是让你知道自己在复制什么。
verdict-retail = 与零售版一致
verdict-patched = 零售版，已应用 4 GB 补丁
verdict-unknown = 本程序不认识的文件，SHA-256 { $sha256 }
verdict-missing = 缺失
detect-unexpected-title = 游戏文件夹中多出的程序
detect-unexpected-help = 零售版 Cry of Fear 并不附带这些。它们通常来自社区补丁。
detect-mods-title = 已知的修改

variant-cof-enhanced = Cry of Fear Enhanced 补丁安装的版本
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = 用 Xash3D 替换引擎
mod-xash-menu = Xash3D 菜单
mod-bunnymod-xt = Bunnymod XT

## 安装位置

install-title = 安装 ReCoF
install-destination = 安装到
install-destination-help = 一个 Steam 之外的新文件夹。ReCoF 需要游戏的完整副本。
install-size-line = 副本需要 { $size }。{ $volume } 上还有 { $available } 可用。
install-space-ok = 空间足够。
install-space-short = { $volume } 上空间不足，还差 { $missing }。
install-space-explain = 除了副本本身，本程序还会为最近几个版本和存档备份预留空间，这样你可以回退，更新也绝不会在没有退路的情况下开始。

dest-inside-steam = 该文件夹在 Steam 文件夹内。ReCoF 必须放在外面，这样 Steam 更新才永远不会覆盖它。
dest-inside-steamapps = 该文件夹在某个 Steam 库的 steamapps 内，那里的文件由 Steam 管理。请另选位置。
dest-protected = 该文件夹需要管理员权限，不适合放游戏。请在你自己的文件里另选位置。
dest-is-vanilla = 那是游戏的 Steam 副本。ReCoF 要装在单独的文件夹里。
dest-drive-root = 那是驱动器的根目录。请在其中选择一个文件夹。
dest-not-empty = 该文件夹里已有 { $entries } 个其他文件。请选择空文件夹或新建一个。
dest-unusable = 该路径无法使用。

## 安装过程

install-copying = 正在复制游戏：{ $current }
install-copying-counted = 正在复制游戏，{ $done } / { $total }
install-applying = 正在应用 ReCoF { $version }
install-finishing = 正在收尾
install-done = ReCoF { $version } 已安装。
install-cancel-confirm = 要停止安装吗？目前已复制的内容都会被删除。你的 Steam 副本不受影响。
install-cancelled = 安装已取消，没有留下任何半成品。
install-offer-import = 从 Steam 副本带来我的存档、设置和解锁内容
install-offer-shortcuts = 在桌面创建快捷方式
install-release-title = 将要安装的内容
install-release-version = ReCoF { $version }
install-release-download = 从 GitHub 下载，并在复制任何文件之前校验其签名。
install-release-stored = 已下载并校验。
install-release-file = 来自 { $name }
install-release-file-help = 复制任何文件之前，会先校验 zip 旁边的签名清单。
install-release-file-clear = 改用已发布的版本
install-release-checking = 正在查找最新版本
install-release-offline = 无法连接 GitHub。请重试，或从文件安装。
install-release-try-beta = 稳定通道上还没有发布任何内容。测试者可以切换到测试版通道。

## 主界面

home-play = 开始 ReCoF
home-play-vanilla = 开始原版
home-version-installed = 已安装 ReCoF { $version }
home-no-install = 尚未安装 ReCoF
home-install-now = 安装 ReCoF
home-channel = 通道：{ $channel }
home-checking = 正在检查更新...
home-up-to-date = 已是最新
home-update-available = ReCoF { $version } 已发布
home-launcher-update-available = 本程序有新版本：{ $version }
home-offline = 无法连接 GitHub。没有网络本程序照样可用。

## 启动游戏

launch-steam-not-running = Steam 没有在运行
launch-steam-not-running-help = Cry of Fear 运行时需要 Steam。请先启动 Steam。
launch-open-steam = 打开 Steam
launch-start-anyway = 仍然启动
launch-starting = 正在启动 ReCoF...
launch-failed = 无法启动 ReCoF。
launch-already-running = ReCoF 已经在运行。

launch-options-title = 启动选项
launch-resolution = 分辨率
launch-resolution-default = 由游戏决定
launch-windowed = 窗口模式
launch-widescreen = 宽屏
launch-3dmenu = 3D 菜单
launch-3dmenu-help = 零售版启动时关闭 3D 菜单。打开后会使用原始的动画菜单。
launch-extra-args = 额外参数
launch-extra-args-help = 原样传给 cof.exe。如果不清楚自己需要什么，请留空。
launch-preview = 命令行

launch-cvars-title = ReCoF 选项
launch-cvars-none = 这个版本的 ReCoF 没有自己的选项。
launch-cvars-from-release = 它们来自你已安装的 ReCoF 版本。
launch-cvars-defaults = 恢复默认值

## 更新

update-title = 更新
update-available-title = ReCoF { $version } 已发布
update-installed-version = 你当前是 { $version }
update-changelog = 更新内容
update-no-changelog = 这个版本没有说明。
update-install = 立即更新
update-later = 稍后
update-downloading = 正在下载 { $name }
update-verifying = 正在校验签名和哈希
update-extracting = 正在解压
update-applying = 正在应用更新
update-done = 已更新到 ReCoF { $version }。
update-backup-first = 先备份你的存档
update-launcher-first = 这个 ReCoF 版本需要 { $needed } 或更新的启动程序。请先更新启动程序。
update-launcher-title = 启动程序更新
update-launcher-restart = 启动程序将重启以完成更新。
update-from-file = 从文件安装
update-from-file-help = 选择一个发行版 zip 包。它的签名清单必须放在同一目录。
changelog-note = 注意
changelog-tip = 提示
changelog-important = 重要
changelog-warning = 警告
changelog-caution = 小心

channel-title = 发行通道
channel-stable = 稳定版
channel-beta = 测试版
channel-beta-help = 测试通道包含预发行版本。请预料到会有问题，并保留你的备份。

## 回退

rollback-title = 回到较早的版本
rollback-none = 没有保留任何较早的版本。
rollback-to = 回到 { $version }
rollback-offer-backup = 恢复那次更新之前备份的存档
rollback-done = 已恢复到 ReCoF { $version }。

## 维护

maintain-title = 维护
maintain-verify = 校验并修复
maintain-verify-help = 按清单计算每个 ReCoF 文件的哈希，并重写所有不匹配的文件。
maintain-backup = 备份存档
maintain-restore = 恢复存档
maintain-import = 从 Steam 副本导入进度
maintain-move = 移动安装位置
maintain-uninstall = 卸载
maintain-shortcuts = 快捷方式
maintain-logs = 日志

verify-running = 正在校验 { $done } / { $total }
verify-intact = 所有文件都匹配。
verify-broken = 有 { $count } 个文件不匹配。
verify-repair = 修复它们
verify-repaired = 已重写 { $count } 个文件。
verify-cannot-repair = 这些文件所属的版本已不在版本库中。请重新安装或更新来修复。

## 存档备份

backup-title = 存档备份
backup-what = 备份内容：游戏存档、设置、解锁内容和你的配置文件。
backup-create = 立即备份
backup-created = 已备份到 { $name }
backup-none = 还没有任何备份。
backup-list-entry = { $when }，{ $size }
backup-restore-confirm = 要恢复 { $name } 吗？你 ReCoF 副本里的存档会被替换。
backup-restored = 备份已恢复。
backup-verify-failed = 备份写入后无法重新读取，因此不可信。没有改动任何东西。
backup-auto = 在更新到 { $version } 之前自动创建

## 从 Steam 副本导入

import-title = 导入进度
import-explain = 把你的存档、设置和解锁内容从 Steam 安装复制到 ReCoF 副本。不会向 Steam 安装写入任何东西，也绝不会反方向复制。
import-nothing-found = Steam 副本里没有可导入的存档。
import-overwrite-title = 这会替换你已有的文件
import-overwrite-body = ReCoF 副本中的 { $count } 个文件将被替换。之前会先做一次备份。
import-done = 已导入 { $count } 个文件。

## 移动

move-title = 移动安装位置
move-to = 移动到
move-running = 正在移动
move-done = 安装位置现在是 { $path }。
move-same-volume = 在同一驱动器上这是瞬间完成的。

## 卸载

uninstall-title = 卸载 ReCoF
uninstall-explain = 这会删除 ReCoF 副本和保存的版本。你的 Steam 安装不受影响。
uninstall-export-first = 先导出我的存档
uninstall-keep-backups = 保留我的存档备份
uninstall-confirm = 要删除 { $path } 吗？
uninstall-done = ReCoF 已删除。

## 快捷方式

shortcut-desktop = 桌面快捷方式
shortcut-start-menu = 开始菜单快捷方式
shortcut-steam = 把 ReCoF 作为非 Steam 游戏添加到 Steam
shortcut-steam-help = 这是本程序唯一会写到自己文件夹之外的操作。Steam 必须关闭，而且之前会先保存一份 shortcuts.vdf 的副本。
shortcut-steam-needs-steam-closed = 请先关闭 Steam。Steam 退出时会重写该文件，从而撤销这次改动。
shortcut-steam-added = ReCoF 已添加到 Steam。重启 Steam 即可看到。
shortcut-created = 快捷方式已创建。

## 报告问题

bug-title = 报告问题
bug-explain = 这里会收集版本号、上次校验的结果、你的系统信息和日志的最后几行。请通读一遍，删掉你不想分享的内容，然后本程序会在浏览器里打开一份预先填好的报告。
bug-never-sends = 不会向任何地方发送任何东西。本程序只是打开你的浏览器。
bug-collecting = 正在收集信息
bug-open-issue = 在浏览器中打开报告
bug-copied = 已复制到剪贴板。

## 设置

settings-title = 设置
settings-language = 语言
settings-check-updates = 启动时检查更新
settings-discord = Discord Rich Presence
settings-discord-help = 游戏运行时在你的 Discord 个人资料上显示你正在玩 ReCoF。默认关闭。
settings-keep-versions = 为回退保留的版本数
settings-open-logs = 打开日志文件夹
settings-about = 关于

## 操作中断后的恢复

recovery-rolled-back = 一次中断的操作已被撤销。安装回到了开始之前的状态。
recovery-rolled-forward = 一次中断的操作已完成。安装现在是版本 { $version }。

## 错误

err-io = 无法读取或写入文件：{ $path }
err-not-supported = 此系统上不支持该功能。
err-registry = 无法读取 Windows 注册表项 { $key }。
err-platform-api = { $api } 调用失败。
err-no-game-exe = { $path } 中没有 cof.exe，因此它不是 Cry of Fear 的文件夹。
err-no-game-dir = { $path } 中没有 cryoffear 文件夹，因此它不是 Cry of Fear 的文件夹。
err-journal = 一次安装操作未能完成。不会留下半成品：本程序会在下次启动时完成或撤销它。
err-interrupted = 操作已停止。

err-no-trusted-keys = 本程序的这个构建没有签名密钥，因此无法校验任何发行版。没有安装任何东西。
err-bad-signature = 这个发行版的签名对不上。没有安装任何东西。请重新下载，如果仍然失败，请报告。
err-bad-manifest = 这个发行版对自身的描述不被本程序接受。没有安装任何东西。
err-launcher-too-old = 这个发行版需要更新的启动程序。请先更新启动程序。
err-wrong-build = 这个发行版是针对另一个 Cry of Fear 版本构建的。
err-hash-mismatch = 下载的文件与预期不符。没有安装任何东西。

err-network = 无法连接 GitHub。请检查网络后重试。
err-github = GitHub 返回了错误：{ $detail }
err-rate-limited = GitHub 正在限制此连接的请求频率。请过几分钟再试。
err-no-release = 这个通道上还没有发布任何版本。
err-command-line-too-long = 命令行将有 { $length } 个字符，而游戏最多只接受 { $limit } 个。请缩短额外参数。
