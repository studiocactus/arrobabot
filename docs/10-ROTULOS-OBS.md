# Rótulos para o OBS e quem está no chat

A tela **Rótulos OBS** gera arquivos `.txt` com totais do canal, eventos recentes, espectadores e horas assistidas, no estilo do StreamLabels. Aponte uma fonte de texto do OBS para um desses arquivos e o número aparece na tela sozinho, sem copiar nada.

## Ativar e escolher a pasta

1. Abra **Rótulos OBS** com o perfil do canal.
2. Escolha a pasta em **Pasta dos arquivos** (ex.: `C:/Live/rotulos`). O BotLive cria a pasta quando preciso.
3. Ligue **Rótulos ativos** e clique em **Salvar rótulos**. Os arquivos são gravados na hora.
4. No OBS, adicione Fonte → Texto → **Ler de arquivo** e aponte para o `.txt` desejado.

Funciona só na Twitch e com o perfil conectado. Os rótulos atualizam nos eventos (follow, sub, cheer, raid) e a cada 30 segundos (espectadores e horas). Totais do canal renovam a cada 5 minutos. **Atualizar agora** força totais, espectadores e horas sem esperar.

## Rótulos disponíveis

| Rótulo | Arquivo | O que sai |
|---|---|---|
| Total de seguidores | follower_total.txt | Total da Twitch |
| Seguidor recente | recent_follower.txt | Nome do último follow |
| Total de subs | sub_total.txt | Total da Twitch |
| Sub recente | recent_sub.txt | Nome da última inscrição |
| Resub recente | recent_resub.txt | Nome e meses |
| Subs de presente na sessão | gifts_session.txt | Quantidade desde que conectou |
| Bits na sessão | cheer_session.txt | Soma de bits desde que conectou |
| Cheer recente | recent_cheer.txt | Nome e bits |
| Raid recente | recent_raid.txt | Nome e espectadores |
| Espectadores no chat | viewers_now.txt | Total com o chat aberto |
| Horas assistidas (todos) | watch_total.txt | Soma de todos os espectadores |
| Quem mais assistiu | top_watcher.txt | Nome e horas |
| Tempo de live | uptime.txt | Horas:minutos:segundos |

Cada rótulo tem interruptor próprio e um **modelo** com tokens (`{name}`, `{total}`, `{amount}`, `{months}`, `{viewers}`, `{count}`, `{hours}`, `{uptime}`). O valor abaixo do modelo mostra o que sai no arquivo agora. Um token fora da lista sai como texto.

## Horas assistidas

Enquanto o perfil está conectado, quem está com o chat aberto soma tempo a cada 30 segundos, mesmo sem falar. O total é acumulado para sempre por pessoa; a sessão (bits e presentes) zera ao conectar. O próprio bot não entra na conta. O cadastro guarda até 20 mil pessoas (as de menor tempo saem).

## Quem está no chat agora

O cartão **Quem está no chat agora** lista quem está com o chat aberto, mesmo em silêncio, com o total. Clique em **Atualizar** para ver a lista. Exige o bot como moderador do canal e **autorizar novamente a conta do bot** no perfil (a permissão de leitura de chatters é nova). Recentes e horas continuam mesmo sem essa permissão; só a lista e a soma por presença param, com aviso no Histórico.
