# Instalação e abertura

## Escolha uma forma de abrir

| Arquivo | Para que serve |
|---|---|
| entrega/BotLive_0.1.0_x64-setup.exe | Instalador Windows x64, com opções de idioma |
| entrega/BotLive_0.1.0_x64_en-US.msi | Instalador alternativo no formato MSI |
| entrega/BotLive.exe | Executável para abrir diretamente, sem instalar o aplicativo |
| INICIAR-BOTLIVE.cmd | Atalho do projeto: procura o executável da entrega ou o build local |
| entrega/SHA256SUMS.txt | Hashes para conferir a integridade dos três binários |

Escolha um instalador; não é necessário instalar o NSIS e o MSI. O executável direto não é um pacote de dados portátil: ele continua usando a pasta de dados do usuário no Windows.

## Requisitos de uso

- Windows x64 e Microsoft Edge WebView2 Runtime.
- Acesso à internet para conectar plataformas, usar IA remota e consultar atualizações.
- Contas e permissões da plataforma desejada.
- Para IA local, Ollama funcionando e um modelo já instalado.
- Para voz local (escuta contínua), o RealtimeSTT na mesma máquina. Ele não acompanha o instalador; veja a seção **Servidor de voz (RealtimeSTT)** abaixo.

O uso do aplicativo compilado não exige Node.js, Rust ou Visual Studio. Essas ferramentas são necessárias apenas para desenvolver ou compilar o código.

O instalador usa o mecanismo de provisionamento do WebView2 do Tauri; uma máquina sem esse runtime pode precisar de internet durante a instalação. O executável direto depende de WebView2 já disponível.

## Instalar e abrir

1. Feche outras instâncias do BotLive.
2. Execute o instalador escolhido.
3. Siga as opções apresentadas por ele.
4. Abra o BotLive pelo atalho instalado.
5. Confirme que aparece a Visão geral, sem a faixa de prévia de navegador.
6. Siga [Primeiro uso](GUIA-DE-USO.md).

A versão distribuída localmente não tem assinatura de um publicador comercial configurada. Confirme a origem do arquivo; este manual não orienta a desativar proteções do Windows.

## Onde os dados ficam

Abra **Configurações → Pasta de dados** e copie o caminho exibido. Ele é a referência para o seu computador. Não use a pasta entrega como origem de backup.

Dentro da pasta de dados ficam o banco botlive.sqlite e o diretório vaults, com uma subpasta por perfil. Credenciais ficam no cofre do sistema operacional, separado desses arquivos.

Duas cópias do aplicativo executadas pelo mesmo usuário podem acessar os mesmos dados e disputar a mesma porta local. Use uma instância por vez.

## Servidor de voz (RealtimeSTT)

A **escuta contínua** precisa do RealtimeSTT rodando na mesma máquina, acessível em `http://127.0.0.1:8010/transcribe-pcm16`. O instalador do BotLive **não** instala o RealtimeSTT — você deve instalá-lo à parte.

Requisitos do RealtimeSTT:
- Python 3.12
- GPU NVIDIA com driver atualizado (o CUDA 12 é preferido; CPU também funciona, mas mais lento)
- Modelos `faster-whisper` baixados na primeira execução (o modelo `small` é o padrão usado pelo painel)

Passos rápidos (PowerShell como administrador):

```powershell
# 1) Instala Python 3.12 se ainda não tiver
winget install --id Python.Python.3.12 -e --accept-package-agreements --accept-source-agreements --disable-interactivity

# 2) Atualiza pip e instala o RealtimeSTT com suporte a servidor e faster-whisper
python -m pip install --upgrade pip
python -m pip install "RealtimeSTT[server,faster-whisper]"
```

Rode o servidor (mantendo a janela aberta ou como serviço):

```powershell
stt-server-production --host 127.0.0.1 --port 8010 --engine faster_whisper --model small --device cuda --compute-type float16 --language pt
```

Opções importantes:
- `--device cuda` exige torch com CUDA (o `pip install torch` padrão traz a versão CPU e o servidor falha). Para CUDA 12, use `pip install torch==2.14.0+cu130 --index-url https://download.pytorch.org/whl/cu130` (ajuste a versão ao seu driver).
- `--compute-type float16` só funciona em GPU; em CPU use `int8` ou remova a opção.
- `--language pt` fixa o idioma; remova para detecção automática.
- O modelo `small` é leve e rápido; `medium` ou `large-v3` dão mais precisão, mas exigem mais VRAM e latência.

Para usar só CPU (sem GPU NVIDIA):

```powershell
stt-server-production --host 127.0.0.1 --port 8010 --engine faster_whisper --model small --device cpu --compute-type int8 --language pt
```

Com o servidor rodando, abra o BotLive, ative o módulo **Controle por voz** e clique em **Abrir Controle por voz** → o campo **Servidor de transcrição** já vem preenchido com `http://127.0.0.1:8010/transcribe-pcm16`. Ligue **Escuta contínua** e autorize o microfone.

## Atualizar, reinstalar e desinstalar

Antes de trocar a versão, feche o aplicativo e faça [backup](07-OPERACAO-E-BACKUP.md). Os dados não ficam junto ao executável, mas preserve uma cópia própria; não dependa do comportamento de um instalador para protegê-los.

A verificação interna de atualizações depende de um endereço de manifesto e de uma chave pública fornecidos pelo distribuidor. Não existe um feed público pronto nesta entrega.

Para atualizar pelo instalador, rode o `.exe` da versão nova por cima da versão instalada. A opção padrão é **não desinstalar**: o executável é substituído no lugar, os atalhos existentes apenas têm o alvo atualizado e o BotLive continua fixado na barra de tarefas. Quando já existe uma instalação, o instalador mostra uma página com as duas escolhas; **desinstalar antes de instalar** apaga também os atalhos e a fixação na barra de tarefas e deve ser usada só quando você quiser começar do zero.

Para desinstalar, use o recurso de aplicativos instalados do Windows. A remoção do programa e a exclusão dos dados são operações diferentes; confira a pasta de dados se quiser conservar ou remover seu espaço.

## Estado dos pacotes

Executável e instaladores Windows foram gerados. A abertura nativa foi verificada no computador de desenvolvimento. A instalação em máquina limpa e os pacotes macOS/Linux ainda não foram homologados. Consulte [Validação](VALIDACAO.md).
