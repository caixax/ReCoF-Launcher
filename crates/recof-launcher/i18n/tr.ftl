# ReCoF Launcher, Türkçe.
#
# İLK TASLAK: bu çeviri henüz ana dili Türkçe olan biri tarafından
# gözden geçirilmedi. Düzeltmeler memnuniyetle karşılanır:
# https://github.com/caixax/ReCoF-Launcher
#
# Biçem: yalın cümleler, uzun çizgi ve orta çizgi kullanılmaz.

app-name = ReCoF Launcher
app-tagline = ReCoF için kurulum ve başlatma aracı

## Her yerde çıkan düğmeler ve sözcükler

button-next = İleri
button-back = Geri
button-cancel = Vazgeç
button-close = Kapat
button-continue = Devam et
button-ok = Tamam
button-yes = Evet
button-no = Hayır
button-browse = Gözat...
button-retry = Yeniden dene
button-copy = Kopyala
button-open-folder = Klasörü aç
button-finish = Bitir
button-skip = Atla
button-details = Ayrıntılar
word-yes = evet
word-no = hayır
word-unknown = bilinmiyor

## İlk açılış: dil

firstrun-language-title = Dilinizi seçin
firstrun-language-help = Bunu sonradan ayarlardan değiştirebilirsiniz.
language-not-reviewed = İlk taslak, ana dili konuşan biri henüz gözden geçirmedi
language-not-reviewed-notice = Bu çeviri bir ilk taslaktır ve ana dili konuşan biri tarafından henüz gözden geçirilmedi. Düzeltmeler memnuniyetle karşılanır.

## İlk açılış: bu nedir

intro-1-title = ReCoF nedir
intro-1-body = ReCoF, Cry of Fear oyununun geliştirilmiş bir yeniden derlemesidir. Bu araç onu kurar, güncel tutar ve çalıştırır. ReCoF ve bu araç, oyunun kendisinden ayrı projelerdir.
intro-2-title = Steam kopyanıza hiç dokunulmaz
intro-2-body = Araç, Steam kurulumunuzu okur ve seçtiğiniz bir klasöre kopyalar. Steam klasörünün içine hiçbir zaman yazmaz, bir şey silmez ve bir şeyin adını değiştirmez, dolayısıyla Cry of Fear tam şu anki gibi çalışmaya devam eder ve istediğiniz zaman ona dönebilirsiniz.
intro-3-title = Ayrı bir kopya ve ilerlemeniz
intro-3-body = ReCoF kendi kopyasında yaşadığı için oyunun tamamı kadar yere ihtiyaç duyar, yaklaşık { $size }. Kayıtlarınız, ayarlarınız ve açtığınız içerikler kurulum sırasında ya da sonradan menüden Steam kopyasından alınabilir. Ters yönde hiçbir şey kopyalanmaz.

## Oyunu bulma

detect-title = Cry of Fear aranıyor
detect-searching = Steam kurulumunuz aranıyor...
detect-found = { $path } konumunda bulundu
detect-not-found = Cry of Fear bulunamadı
detect-not-found-help = Steam kurulu olmayabilir ya da oyun aracın göremediği bir klasörde olabilir. cof.exe dosyasının bulunduğu klasörü seçin.
detect-pick-folder = Oyun klasörünü seç
detect-pick-folder-help = cof.exe dosyasının bulunduğu klasör
detect-library = Steam kitaplığı: { $path }
detect-buildid-ok = Build { $buildid }, ReCoF tam bunun için derlendi
detect-buildid-wrong = Bu build { $found }. ReCoF build { $expected } için derlendi. Düzgün çalışmayabilir ve sorun çıkması beklenir.
detect-buildid-unknown = Steam build numarası okunamadı. Elle seçilen bir klasörde bu normaldir.
detect-size = { $files } dosya, { $size }

## Dosyalar ne çıktı

detect-check-title = Oyun dosyaları denetleniyor
detect-check-clean = Bütün dosyalar 1.6a perakende sürümüyle uyuşuyor.
detect-check-notices = Bazı dosyalar 1.6a perakende sürümünün getirdikleri değil. Bu kurulumu engellemez: kopya sizindir ve ReCoF o dosyaları zaten değiştirir. Neyi kopyaladığınızı bilin diye burada yazıyor.
verdict-retail = perakende sürümle uyuşuyor
verdict-patched = perakende sürüm, 4 GB yaması uygulanmış
verdict-unknown = aracın tanımadığı bir dosya, SHA-256 { $sha256 }
verdict-missing = eksik
detect-unexpected-title = Oyun klasöründe fazladan programlar
detect-unexpected-help = Cry of Fear perakende sürümü bunları içermez. Genellikle bir topluluk yamasından gelirler.
detect-mods-title = Bilinen değişiklikler

variant-cof-enhanced = Cry of Fear Enhanced yamasının kurduğu sürüm
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = motorun Xash3D ile değiştirilmesi
mod-xash-menu = Xash3D menüsü
mod-bunnymod-xt = Bunnymod XT

## Nereye kurulacak

install-title = ReCoF kur
install-destination = Şuraya kur
install-destination-help = Steam dışında yeni bir klasör. ReCoF oyunun tam bir kopyasına ihtiyaç duyar.
install-size-line = Kopya { $size } yer ister. { $volume } sürücüsünde { $available } boş.
install-space-ok = Yer var.
install-space-short = { $volume } sürücüsünde yeterli yer yok. { $missing } eksik.
install-space-explain = Kopyanın yanı sıra araç, son birkaç sürüm ve kayıt yedekleri için yer ayırır, böylece geri dönebilirsiniz ve bir güncelleme hiçbir zaman dönüşü olmadan başlamaz.

dest-inside-steam = O klasör Steam klasörünün içinde. ReCoF dışarıda durmalı ki bir Steam güncellemesi onun üzerine hiçbir zaman yazamasın.
dest-inside-steamapps = O klasör bir Steam kitaplığının steamapps klasörünün içinde, dosyaları orada Steam yönetir. Başka bir yer seçin.
dest-protected = O klasör yönetici yetkisi ister ve bir oyun için iyi bir yer değildir. Kendi dosyalarınızın içinden bir yer seçin.
dest-is-vanilla = Burası oyunun Steam kopyası. ReCoF ayrı bir klasöre kurulur.
dest-drive-root = Burası bir sürücünün kökü. İçinde bir klasör seçin.
dest-not-empty = O klasörde zaten başka { $entries } dosya var. Boş ya da yeni bir klasör seçin.
dest-unusable = Bu yol kullanılamaz.

## Kurulum

install-copying = Oyun kopyalanıyor: { $current }
install-copying-counted = Oyun kopyalanıyor, { $total } dosyadan { $done }
install-applying = ReCoF { $version } uygulanıyor
install-finishing = Tamamlanıyor
install-done = ReCoF { $version } kuruldu.
install-cancel-confirm = Kurulum durdurulsun mu? Şimdiye kadar kopyalanan her şey silinecek. Steam kopyanız etkilenmez.
install-cancelled = Kurulum iptal edildi ve geride hiçbir şey kalmadı.
install-offer-import = Kayıtlarımı, ayarlarımı ve açtıklarımı Steam kopyasından getir
install-offer-shortcuts = Masaüstünde kısayol oluştur
install-release-title = Ne kurulacak
install-release-version = ReCoF { $version }
install-release-download = GitHub'dan indirilir ve herhangi bir şey kopyalanmadan önce imzasıyla doğrulanır.
install-release-stored = Zaten indirildi ve doğrulandı.
install-release-file = Kaynak: { $name }
install-release-file-help = Zip dosyasının yanındaki imzalı manifest, herhangi bir şey kopyalanmadan önce doğrulanır.
install-release-file-clear = Bunun yerine yayımlanmış bir sürüm kullan
install-release-checking = En son sürüm aranıyor
install-release-offline = GitHub'a ulaşılamadı. Yeniden deneyin veya bir dosyadan kurun.
install-release-try-beta = Kararlı kanalda henüz bir şey yayımlanmadı. Test edenler betaya geçebilir.

## Ana ekran

home-play = ReCoF oyna
home-play-vanilla = Özgün sürümü oyna
home-version-installed = ReCoF { $version } kurulu
home-no-install = ReCoF henüz kurulu değil
home-install-now = ReCoF kur
home-channel = Kanal: { $channel }
home-checking = Güncellemeler aranıyor...
home-up-to-date = Her şey güncel
home-update-available = ReCoF { $version } yayımlandı
home-launcher-update-available = Aracın yeni bir sürümü var: { $version }
home-offline = GitHub'a ulaşılamadı. Araç bunsuz da çalışır.

## Oyunu başlatma

launch-steam-not-running = Steam çalışmıyor
launch-steam-not-running-help = Cry of Fear çalışırken Steam'i kullanır. Önce Steam'i açın.
launch-open-steam = Steam'i aç
launch-start-anyway = Yine de başlat
launch-starting = ReCoF başlatılıyor...
launch-failed = ReCoF başlatılamadı.
launch-already-running = ReCoF zaten çalışıyor.

launch-options-title = Başlatma seçenekleri
launch-resolution = Çözünürlük
launch-resolution-default = Oyunun seçtiği
launch-windowed = Pencerede
launch-widescreen = Geniş ekran
launch-3dmenu = 3B menü
launch-3dmenu-help = Perakende sürüm 3B menü kapalı olarak başlar. Açarsanız özgün hareketli menü kullanılır.
launch-extra-args = Ek argümanlar
launch-extra-args-help = cof.exe dosyasına olduğu gibi geçirilir. Neye ihtiyacınız olduğunu bilmiyorsanız boş bırakın.
launch-preview = Komut satırı

launch-cvars-title = ReCoF seçenekleri
launch-cvars-none = ReCoF'un bu sürümü kendine ait seçenek getirmiyor.
launch-cvars-from-release = Kurulu olan ReCoF sürümünden gelirler.
launch-cvars-defaults = Varsayılanlara dön

## Güncellemeler

update-title = Güncelle
update-available-title = ReCoF { $version } yayımlandı
update-installed-version = Sizde { $version } var
update-changelog = Neler değişti
update-no-changelog = Bu sürümün notu yok.
update-install = Şimdi güncelle
update-later = Sonra
update-downloading = { $name } indiriliyor
update-verifying = İmza ve özetler denetleniyor
update-extracting = Açılıyor
update-applying = Güncelleme uygulanıyor
update-done = ReCoF { $version } sürümüne güncellendi.
update-backup-first = Önce kayıtlarınızın yedeği alınıyor
update-launcher-first = ReCoF'un bu sürümü { $needed } ya da daha yeni bir araç ister. Önce aracı güncelleyin.
update-launcher-title = Araç güncellemesi
update-launcher-restart = Güncellemeyi bitirmek için araç yeniden başlayacak.
update-from-file = Bir dosyadan kur
update-from-file-help = Bir sürüm zip dosyası seçin. İmzalı manifest dosyası yanında olmalı.
changelog-note = Not
changelog-tip = İpucu
changelog-important = Önemli
changelog-warning = Uyarı
changelog-caution = Dikkat

channel-title = Yayın kanalı
channel-stable = Kararlı
channel-beta = Beta
channel-beta-help = Beta kanalı ön sürümleri içerir. Sorun çıkmasını bekleyin ve yedeklerinizi saklayın.

## Geri dönme

rollback-title = Daha eski bir sürüme dön
rollback-none = Saklanan daha eski bir sürüm yok.
rollback-to = { $version } sürümüne dön
rollback-offer-backup = O güncellemeden önce yedeklenen kayıtları geri yükle
rollback-done = ReCoF { $version } geri yüklendi.

## Bakım

maintain-title = Bakım
maintain-verify = Denetle ve onar
maintain-verify-help = Her ReCoF dosyasının özetini manifestle karşılaştırır ve uyuşmayanı yeniden yazar.
maintain-backup = Kayıtları yedekle
maintain-restore = Kayıtları geri yükle
maintain-import = İlerlemeyi Steam kopyasından al
maintain-move = Kurulumu taşı
maintain-uninstall = Kaldır
maintain-shortcuts = Kısayollar
maintain-logs = Günlükler

verify-running = { $total } dosyadan { $done } denetleniyor
verify-intact = Bütün dosyalar uyuşuyor.
verify-broken = { $count } dosya uyuşmuyor.
verify-repair = Onar
verify-repaired = { $count } dosya yeniden yazıldı.
verify-cannot-repair = Bu dosyaların geldiği sürüm artık depoda yok. Onarmak için yeniden kurun ya da güncelleyin.

## Kayıt yedekleri

backup-title = Kayıt yedekleri
backup-what = Yedeklenenler: oyun kayıtları, ayarlar, açılan içerikler ve yapılandırma dosyalarınız.
backup-create = Şimdi yedekle
backup-created = { $name } olarak yedeklendi
backup-none = Henüz hiç yedek yok.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = { $name } geri yüklensin mi? ReCoF kopyanızdaki kayıtların üzerine yazılacak.
backup-restored = Yedek geri yüklendi.
backup-verify-failed = Yedek, yazıldıktan sonra geri okunamadı, bu yüzden güvenilir değil. Hiçbir şey değiştirilmedi.
backup-auto = { $version } sürümüne güncellemeden önce otomatik alındı

## Steam kopyasından alma

import-title = İlerlemeyi al
import-explain = Kayıtlarınızı, ayarlarınızı ve açtığınız içerikleri Steam kurulumundan ReCoF kopyasına kopyalar. Steam kurulumunun içine hiçbir şey yazılmaz ve ters yönde hiçbir şey kopyalanmaz.
import-nothing-found = Steam kopyasında alınacak kayıt yok.
import-overwrite-title = Bu, hâlihazırda bulunan dosyaların üzerine yazacak
import-overwrite-body = ReCoF kopyasındaki { $count } dosyanın üzerine yazılacak. Önce yedek alınır.
import-done = { $count } dosya alındı.

## Taşıma

move-title = Kurulumu taşı
move-to = Şuraya taşı
move-running = Taşınıyor
move-done = Kurulum artık { $path } konumunda.
move-same-volume = Aynı sürücüde bu anında olur.

## Kaldırma

uninstall-title = ReCoF'u kaldır
uninstall-explain = Bu, ReCoF kopyasını ve saklanan sürümleri siler. Steam kurulumunuza dokunulmaz.
uninstall-export-first = Önce kayıtlarımı dışa aktar
uninstall-keep-backups = Kayıt yedeklerimi sakla
uninstall-confirm = { $path } silinsin mi?
uninstall-done = ReCoF silindi.

## Kısayollar

shortcut-desktop = Masaüstü kısayolu
shortcut-start-menu = Başlat menüsü kısayolu
shortcut-steam = ReCoF'u Steam'e Steam dışı oyun olarak ekle
shortcut-steam-help = Aracın kendi klasörlerinin dışına yazdığı tek şey budur. Steam kapalı olmalı ve önce shortcuts.vdf dosyasının bir kopyası saklanır.
shortcut-steam-needs-steam-closed = Önce Steam'i kapatın. Steam çıkarken bu dosyayı yeniden yazar ve değişikliği geri alır.
shortcut-steam-added = ReCoF Steam'e eklendi. Görmek için Steam'i yeniden başlatın.
shortcut-created = Kısayol oluşturuldu.

## Hata bildirme

bug-title = Hata bildir
bug-explain = Burada sürümler, son denetimin sonucu, sisteminizin bilgileri ve günlüğün son satırları toplanır. Hepsini gözden geçirin, paylaşmak istemediğinizi çıkarın, araç tarayıcınızda önceden doldurulmuş bir bildirim açsın.
bug-never-sends = Hiçbir yere hiçbir şey gönderilmez. Araç yalnızca tarayıcınızı açar.
bug-collecting = Bilgiler toplanıyor
bug-open-issue = Bildirimi tarayıcımda aç
bug-copied = Panoya kopyalandı.

## Ayarlar

settings-title = Ayarlar
settings-language = Dil
settings-check-updates = Araç açılırken güncellemeleri ara
settings-discord = Discord Rich Presence
settings-discord-help = Oyun açıkken Discord profilinizde ReCoF oynadığınızı gösterir. Varsayılan olarak kapalıdır.
settings-keep-versions = Geri dönmek için saklanan sürüm sayısı
settings-open-logs = Günlük klasörünü aç
settings-about = Hakkında

## Yarıda kalan bir işlemden sonra kurtarma

recovery-rolled-back = Yarıda kalan bir işlem geri alındı. Kurulum, başlamadan önceki hâlinde.
recovery-rolled-forward = Yarıda kalan bir işlem tamamlandı. Kurulum { $version } sürümünde.

## Hatalar

err-io = Bir dosya okunamadı ya da yazılamadı: { $path }
err-not-supported = Bu, bu sistemde kullanılamıyor.
err-registry = Windows kayıt defteri anahtarı { $key } okunamadı.
err-platform-api = { $api } başarısız oldu.
err-no-game-exe = { $path } içinde cof.exe yok, dolayısıyla burası bir Cry of Fear klasörü değil.
err-no-game-dir = { $path } içinde cryoffear klasörü yok, dolayısıyla burası bir Cry of Fear klasörü değil.
err-journal = Bir kurulum işlemi tamamlanamadı. Hiçbir şey yarım kalmaz: araç bir sonraki açılışta onu tamamlar ya da geri alır.
err-interrupted = İşlem durduruldu.

err-no-trusted-keys = Aracın bu derlemesinde imza anahtarı yok, bu yüzden hiçbir sürümü doğrulayamaz. Hiçbir şey kurulmadı.
err-bad-signature = Bu sürümün imzası tutmuyor. Hiçbir şey kurulmadı. Yeniden indirin, yine olmazsa bildirin.
err-bad-manifest = Bu sürüm kendini aracın kabul etmediği bir biçimde tanımlıyor. Hiçbir şey kurulmadı.
err-launcher-too-old = Bu sürüm daha yeni bir araç ister. Önce aracı güncelleyin.
err-wrong-build = Bu sürüm Cry of Fear'ın başka bir sürümü için derlenmiş.
err-hash-mismatch = İndirilen bir dosya olması gereken dosya değil. Hiçbir şey kurulmadı.

err-network = GitHub'a ulaşılamadı. Bağlantınızı denetleyip yeniden deneyin.
err-github = GitHub bir hatayla yanıt verdi: { $detail }
err-rate-limited = GitHub bu bağlantının isteklerini kısıtlıyor. Birkaç dakika sonra yeniden deneyin.
err-no-release = Bu kanalda henüz hiçbir sürüm yayımlanmadı.
err-command-line-too-long = Komut satırı { $length } karakter olurdu, oyun en fazla { $limit } karakter kabul ediyor. Ek bağımsız değişkenleri kısaltın.

## Ayarlar, sekmeler hâlinde

settings-tab-look = Görünüm
settings-tab-language = Dil
settings-tab-updates = Güncellemeler
settings-tab-behaviour = Davranış
settings-tab-about = Hakkında

## Görünüm

look-title = Görünüm
look-show-hero = Arka plan görselini kullan
look-hero-dim = Arka plan ne kadar koyultulsun
look-hero-dim-help = Üstündeki yazı okunur kalacak kadar.
look-grain = Film greni
look-command-line = Komut satırını ana ekranda göster

look-artwork-title = Kendi görselleriniz
look-artwork-help = Araç oyunun hiçbir görselini içermez. Kendi görsellerinizi bu klasöre koyun, onları kullanır: arka plan için hero.png, başlık için logo.png, pencere simgesi için icon.png. Bu makinede kalırlar ve hiçbir sürüme girmezler.
look-open-theme = Görsel klasörünü aç
look-reload-art = Görselleri yeniden yükle
look-using-art = Görselleriniz kullanılıyor: { $pieces }
look-no-art = Görsel bulunamadı, bu yüzden araç kendi çizdiklerini kullanıyor.

## Davranış

behaviour-minimize = Oyun başlayınca aracı simge durumuna küçült
behaviour-close = Oyun başlayınca aracı kapat
behaviour-close-help = Simge durumuna küçültmeye göre önceliklidir.

## Hakkında

about-what = ReCoF Launcher, Cry of Fear'ın geliştirilmiş yeniden derlemesi olan ReCoF'u kurar, günceller ve çalıştırır. Steam kurulumunuzu seçtiğiniz bir klasöre kopyalar ve Steam'inkinin içine asla yazmaz.
about-made-by = Yapan: { $author }, ReCoF'un kendisini de o yapıyor.
about-version = Araç { $version }
about-licence = MIT lisanslı. Kaynak kodu { $url } adresinde.
about-recof = ReCoF { $url } adresinde.
about-font = Başlık, SIL Open Font License 1.1 altındaki Archivo Black ile dizildi.
about-not-affiliated = Team Psykskallar ile bir ilişkisi yoktur. Cry of Fear yazarlarına aittir; bu araç yalnızca kendi kopyanızdan yapılmış bir yeniden derlemeyi kurar.

## Görsel künyesi

look-credits-title = Görsel künyesi
look-credits-none = Bu görseller künyesiz geldi.
look-credits-help = Görseller aracın bir parçası değildir. ReCoF sürümünden gelirler ve onları yapanlara aittir.
look-forget-art = Görselleri kaldır
