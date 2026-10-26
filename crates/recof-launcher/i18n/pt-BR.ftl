# ReCoF Launcher, português do Brasil.
#
# PRIMEIRO RASCUNHO: esta tradução ainda não foi revisada por uma pessoa
# falante nativa. Correções são muito bem-vindas:
# https://github.com/caixax/ReCoF-Launcher
#
# Estilo: frases simples, sem travessões nem meias-riscas.

app-name = ReCoF Launcher
app-tagline = Instalador e inicializador do ReCoF

## Botões e palavras que aparecem em todo lugar

button-next = Avançar
button-back = Voltar
button-cancel = Cancelar
button-close = Fechar
button-continue = Continuar
button-ok = OK
button-yes = Sim
button-no = Não
button-browse = Procurar...
button-retry = Tentar de novo
button-copy = Copiar
button-open-folder = Abrir pasta
button-finish = Concluir
button-skip = Pular
button-details = Detalhes
word-yes = sim
word-no = não
word-unknown = desconhecido

## Primeira execução: idioma

firstrun-language-title = Escolha seu idioma
firstrun-language-help = Você pode mudar isso depois nas configurações.
language-not-reviewed = Primeiro rascunho, ainda sem revisão de falante nativo
language-not-reviewed-notice = Esta tradução é um primeiro rascunho e ainda não foi revisada por uma pessoa falante nativa. Correções são bem-vindas.

## Primeira execução: do que se trata

intro-1-title = O que é o ReCoF
intro-1-body = O ReCoF é uma recompilação melhorada de Cry of Fear. Este inicializador o instala, mantém atualizado e o executa. O ReCoF e este inicializador são projetos separados do jogo em si.
intro-2-title = Sua cópia do Steam nunca é tocada
intro-2-body = O inicializador lê sua instalação do Steam e a copia para uma pasta que você escolher. Ele nunca escreve, apaga nem renomeia nada dentro da pasta do Steam, então Cry of Fear continua funcionando exatamente como agora e você pode voltar para ele quando quiser.
intro-3-title = Uma cópia à parte, e o seu progresso
intro-3-body = Como o ReCoF vive na própria cópia, ele precisa de espaço para o jogo inteiro, cerca de { $size }. Seus saves, configurações e desbloqueios podem ser trazidos da cópia do Steam durante a instalação, ou mais tarde pelo menu. Nada é copiado no sentido contrário.

## Encontrar o jogo

detect-title = Procurando Cry of Fear
detect-searching = Procurando sua instalação do Steam...
detect-found = Encontrado em { $path }
detect-not-found = Cry of Fear não foi encontrado
detect-not-found-help = Talvez o Steam não esteja instalado, ou o jogo esteja em uma pasta que o inicializador não enxerga. Escolha a pasta que contém cof.exe.
detect-pick-folder = Escolher a pasta do jogo
detect-pick-folder-help = A pasta que contém cof.exe
detect-library = Biblioteca do Steam: { $path }
detect-buildid-ok = Build { $buildid }, a que o ReCoF espera
detect-buildid-wrong = Esta é a build { $found }. O ReCoF foi feito para a build { $expected }. Pode não funcionar direito, e é de esperar que dê problemas.
detect-buildid-unknown = Não foi possível ler a build do Steam. Isso é normal para uma pasta escolhida à mão.
detect-size = { $files } arquivos, { $size }

## O que os arquivos acabaram sendo

detect-check-title = Conferindo os arquivos do jogo
detect-check-clean = Todos os arquivos conferem com a versão comercial 1.6a.
detect-check-notices = Alguns arquivos não são os da versão comercial 1.6a. Isso não impede a instalação: a cópia é sua e o ReCoF substitui esses arquivos de qualquer jeito. Está listado aqui para você saber o que está copiando.
verdict-retail = confere com a versão comercial
verdict-patched = versão comercial, com o patch de 4 GB aplicado
verdict-unknown = não é um arquivo que o inicializador conheça, SHA-256 { $sha256 }
verdict-missing = faltando
detect-unexpected-title = Programas a mais na pasta do jogo
detect-unexpected-help = A versão comercial de Cry of Fear não os inclui. Costumam vir de um patch da comunidade.
detect-mods-title = Modificações conhecidas

variant-cof-enhanced = a versão que o patch Cry of Fear Enhanced instala
mod-cof-enhanced = Cry of Fear Enhanced
mod-xash = substituição do motor por Xash3D
mod-xash-menu = menu do Xash3D
mod-bunnymod-xt = Bunnymod XT

## Onde instalar

install-title = Instalar o ReCoF
install-destination = Instalar em
install-destination-help = Uma pasta nova, fora do Steam. O ReCoF precisa de uma cópia completa do jogo.
install-size-line = A cópia precisa de { $size }. Em { $volume } há { $available } livres.
install-space-ok = Há espaço.
install-space-short = Não há espaço suficiente em { $volume }. Faltam { $missing }.
install-space-explain = Além da cópia, o inicializador reserva espaço para as últimas versões e para os backups de saves, para que você possa voltar atrás e para que uma atualização nunca comece sem saída.

dest-inside-steam = Essa pasta está dentro da pasta do Steam. O ReCoF precisa ficar fora, para que uma atualização do Steam nunca possa sobrescrevê-lo.
dest-inside-steamapps = Essa pasta está dentro do steamapps de uma biblioteca do Steam, onde o Steam gerencia os arquivos. Escolha outro lugar.
dest-protected = Essa pasta exige direitos de administrador e não é um bom lugar para um jogo. Escolha algo dentro dos seus próprios arquivos.
dest-is-vanilla = Essa é a cópia do jogo no Steam. O ReCoF vai em uma pasta separada.
dest-drive-root = Essa é a raiz de um disco. Escolha uma pasta dentro dele.
dest-not-empty = Essa pasta já contém { $entries } arquivos de outra coisa. Escolha uma pasta vazia ou uma nova.
dest-unusable = Esse caminho não pode ser usado.

## Instalando

install-copying = Copiando o jogo: { $current }
install-copying-counted = Copiando o jogo, { $done } de { $total }
install-applying = Aplicando o ReCoF { $version }
install-finishing = Finalizando
install-done = O ReCoF { $version } está instalado.
install-cancel-confirm = Parar a instalação? Tudo o que foi copiado até agora será apagado. Sua cópia do Steam não é afetada.
install-cancelled = A instalação foi cancelada e não deixou nada pela metade.
install-offer-import = Trazer meus saves, configurações e desbloqueios da cópia do Steam
install-offer-shortcuts = Criar um atalho na área de trabalho

## A tela principal

home-play = Jogar ReCoF
home-play-vanilla = Jogar o original
home-version-installed = ReCoF { $version } instalado
home-no-install = O ReCoF ainda não está instalado
home-install-now = Instalar o ReCoF
home-channel = Canal: { $channel }
home-checking = Procurando atualizações...
home-up-to-date = Tudo em dia
home-update-available = O ReCoF { $version } está disponível
home-launcher-update-available = Há uma versão nova do inicializador: { $version }
home-offline = Não foi possível acessar o GitHub. O inicializador funciona mesmo assim.

## Iniciar o jogo

launch-steam-not-running = O Steam não está aberto
launch-steam-not-running-help = Cry of Fear usa o Steam enquanto roda. Abra o Steam primeiro.
launch-open-steam = Abrir o Steam
launch-start-anyway = Iniciar mesmo assim
launch-starting = Iniciando o ReCoF...
launch-failed = Não foi possível iniciar o ReCoF.
launch-already-running = O ReCoF já está em execução.

launch-options-title = Opções de inicialização
launch-resolution = Resolução
launch-resolution-default = A que o jogo decidir
launch-windowed = Em janela
launch-widescreen = Tela panorâmica
launch-3dmenu = Menu 3D
launch-3dmenu-help = A versão comercial começa com o menu 3D desligado. Ligando-o, usa-se o menu animado original.
launch-extra-args = Argumentos extras
launch-extra-args-help = Passados para o cof.exe como estão. Deixe vazio se não souber do que precisa.
launch-preview = Linha de comando

launch-cvars-title = Opções do ReCoF
launch-cvars-none = Esta versão do ReCoF não traz opções próprias.
launch-cvars-from-release = Vêm da versão do ReCoF que você tem instalada.

## Atualizações

update-title = Atualizar
update-available-title = O ReCoF { $version } está disponível
update-installed-version = Você tem a { $version }
update-changelog = O que mudou
update-no-changelog = Esta versão não tem notas.
update-install = Atualizar agora
update-later = Depois
update-downloading = Baixando { $name }
update-verifying = Conferindo a assinatura e os hashes
update-extracting = Descompactando
update-applying = Aplicando a atualização
update-done = Atualizado para o ReCoF { $version }.
update-backup-first = Fazendo primeiro um backup dos seus saves
update-launcher-first = Esta versão do ReCoF precisa do inicializador { $needed } ou mais novo. Atualize o inicializador primeiro.
update-launcher-title = Atualização do inicializador
update-launcher-restart = O inicializador vai reiniciar para terminar a atualização.
update-from-file = Instalar a partir de um arquivo
update-from-file-help = Escolha um zip de versão. O manifesto assinado precisa estar ao lado dele.

channel-title = Canal de versões
channel-stable = Estável
channel-beta = Beta
channel-beta-help = O canal beta inclui pré-lançamentos. Espere problemas e guarde seus backups.

## Voltar atrás

rollback-title = Voltar para uma versão anterior
rollback-none = Nenhuma versão anterior foi guardada.
rollback-to = Voltar para { $version }
rollback-offer-backup = Restaurar os saves guardados antes daquela atualização
rollback-done = O ReCoF { $version } foi restaurado.

## Manutenção

maintain-title = Manutenção
maintain-verify = Verificar e reparar
maintain-verify-help = Calcula o hash de cada arquivo do ReCoF contra o manifesto e reescreve o que não bater.
maintain-backup = Backup dos saves
maintain-restore = Restaurar saves
maintain-import = Importar progresso da cópia do Steam
maintain-move = Mover a instalação
maintain-uninstall = Desinstalar
maintain-shortcuts = Atalhos
maintain-logs = Registros

verify-running = Conferindo { $done } de { $total }
verify-intact = Todos os arquivos batem.
verify-broken = { $count } arquivos não batem.
verify-repair = Repará-los
verify-repaired = { $count } arquivos foram reescritos.
verify-cannot-repair = A versão de onde esses arquivos vieram não está mais guardada. Reinstale ou atualize para reparar.

## Backups dos saves

backup-title = Backups dos saves
backup-what = São guardados: saves, configurações, desbloqueios e seus arquivos de configuração.
backup-create = Fazer backup agora
backup-created = Guardado em { $name }
backup-none = Ainda não há nenhum backup.
backup-list-entry = { $when }, { $size }
backup-restore-confirm = Restaurar { $name }? Os saves da sua cópia do ReCoF serão substituídos.
backup-restored = O backup foi restaurado.
backup-verify-failed = Não foi possível reler o backup depois de escrevê-lo, então ele não é confiável. Nada foi alterado.
backup-auto = Feito automaticamente antes de atualizar para { $version }

## Importar da cópia do Steam

import-title = Importar progresso
import-explain = Copia seus saves, configurações e desbloqueios da instalação do Steam para a cópia do ReCoF. Nada é escrito dentro da instalação do Steam, e nada é copiado no sentido contrário.
import-nothing-found = A cópia do Steam não tem saves para importar.
import-overwrite-title = Isto vai substituir arquivos que você já tem
import-overwrite-body = { $count } arquivos da cópia do ReCoF serão substituídos. Antes é feito um backup.
import-done = { $count } arquivos foram importados.

## Mover

move-title = Mover a instalação
move-to = Mover para
move-running = Movendo
move-done = A instalação agora está em { $path }.
move-same-volume = No mesmo disco isso é imediato.

## Desinstalar

uninstall-title = Desinstalar o ReCoF
uninstall-explain = Isto remove a cópia do ReCoF e as versões guardadas. Sua instalação do Steam não é tocada.
uninstall-export-first = Exportar meus saves antes
uninstall-keep-backups = Manter meus backups de saves
uninstall-confirm = Remover { $path }?
uninstall-done = O ReCoF foi removido.

## Atalhos

shortcut-desktop = Atalho na área de trabalho
shortcut-start-menu = Atalho no menu Iniciar
shortcut-steam = Adicionar o ReCoF ao Steam como jogo externo
shortcut-steam-help = É a única coisa que o inicializador escreve fora das próprias pastas. O Steam precisa estar fechado, e antes é guardada uma cópia do shortcuts.vdf.
shortcut-steam-needs-steam-closed = Feche o Steam primeiro. O Steam reescreve esse arquivo ao sair, o que desfaria a mudança.
shortcut-steam-added = O ReCoF foi adicionado ao Steam. Reinicie o Steam para vê-lo.
shortcut-created = Atalho criado.

## Relatar um problema

bug-title = Relatar um problema
bug-explain = Isto reúne as versões, o resultado da última verificação, os dados do seu sistema e as últimas linhas do registro. Revise tudo, tire o que preferir não compartilhar, e o inicializador abre no seu navegador um relato já preenchido.
bug-never-sends = Nada é enviado para lugar nenhum. O inicializador só abre o seu navegador.
bug-collecting = Reunindo os dados
bug-open-issue = Abrir o relato no meu navegador
bug-copied = Copiado para a área de transferência.

## Configurações

settings-title = Configurações
settings-language = Idioma
settings-check-updates = Procurar atualizações ao abrir o inicializador
settings-discord = Discord Rich Presence
settings-discord-help = Mostra no seu perfil do Discord que você está jogando ReCoF, enquanto o jogo está aberto. Desligado por padrão.
settings-keep-versions = Versões guardadas para voltar atrás
settings-open-logs = Abrir a pasta de registros
settings-about = Sobre

## Recuperação depois de uma operação interrompida

recovery-rolled-back = Uma operação interrompida foi desfeita. A instalação está como estava antes de começar.
recovery-rolled-forward = Uma operação interrompida foi concluída. A instalação está na versão { $version }.

## Erros

err-io = Não foi possível ler ou escrever um arquivo: { $path }
err-not-supported = Isto não está disponível neste sistema.
err-registry = Não foi possível ler a chave de registro do Windows { $key }.
err-platform-api = { $api } falhou.
err-no-game-exe = { $path } não contém cof.exe, então não é uma pasta de Cry of Fear.
err-no-game-dir = { $path } não contém uma pasta cryoffear, então não é uma pasta de Cry of Fear.
err-journal = Não foi possível concluir uma operação de instalação. Nada fica pela metade: o inicializador vai concluí-la ou desfazê-la na próxima vez que abrir.
err-interrupted = A operação foi interrompida.

err-no-trusted-keys = Esta compilação do inicializador não tem chave de assinatura, então não consegue verificar nenhuma versão. Nada foi instalado.
err-bad-signature = A assinatura desta versão não confere. Nada foi instalado. Baixe de novo, e se continuar falhando, relate.
err-bad-manifest = Esta versão se descreve de um jeito que o inicializador não aceita. Nada foi instalado.
err-launcher-too-old = Esta versão precisa de um inicializador mais novo. Atualize o inicializador primeiro.
err-wrong-build = Esta versão foi feita para outra versão de Cry of Fear.
err-hash-mismatch = Um arquivo baixado não é o que deveria ser. Nada foi instalado.

err-network = Não foi possível acessar o GitHub. Verifique sua conexão e tente de novo.
err-github = O GitHub respondeu com um erro: { $detail }
err-rate-limited = O GitHub está limitando as requisições desta conexão. Tente de novo em alguns minutos.
err-no-release = Ainda não foi publicada nenhuma versão neste canal.
