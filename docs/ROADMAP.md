# Roadmap

O que já está no BotLive e o que está planejado. Este documento é a referência central de direção; o histórico de entregas continua em [Atualizações e commits](ATUALIZACOES.md).

## Entregue

- **UX 2.0 (em andamento)** — as primeiras etapas já estão na versão atual:
  - Resumo legível de cada etapa direto no bloco do canvas, com a cadeia empilhada de cima para baixo.
  - **Adicionar etapa** cria a etapa no fim da cadeia já ligada à última, selecionada e com a configuração aberta, em espaço livre e sem redefinir o zoom.
  - **Inserir etapa depois** (menu **•••**) encaixa uma etapa no meio da sequência (A → nova → B), preservando os dados das vizinhas e o eixo do desenho.
  - Menu **Opções da etapa** com **Subir/Descer** por conexões, limites por posição na cadeia e devolução de foco ao fechar.
  - Comportamento de fluxos inválidos (desconectado, cíclico ou com ramificação): o editor explica e não reorganiza nada em silêncio.
  - Minimapa compacto e ajuda do editor em popover, com preferência preservada.
  - Painel **Testar fluxo** reformado: abre nos dois editores sem salvar nem executar, mostra o que o teste faz e os valores de exemplo com o nome legível de cada variável, começa uma única execução por clique (o duplo não duplica), acompanha cada etapa com o estado traduzido — parada por condição não é tratada como falha — e cancela ou fecha durante a execução só depois da sua escolha, sem efeito do lado de fora.

> A etapa de UX 2.0 segue em andamento: este documento não declara o programa concluído. Próximas entregas serão listadas aqui conforme forem aprovadas.

## Planejado

### Integração com o Voicemod (PLANEJADO)

Trocar a voz do bot pelo microfone virtual do Voicemod durante falas em TTS, usando a [Voicemod Control API](https://control-api.voicemod.net/). A intenção é provar compatibilidade antes de prometer a funcionalidade: a implementação só começa depois que a API for validada de verdade, com o aplicativo Voicemod aberto.

Fases previstas:

1. **Validação da Control API** — abrir a documentação oficial, descobrir porta e endereços, e confirmar por experimento que dá para consultar e trocar o microfone virtual com o Voicemod rodando.
2. **Conexão e estado** — conectar, manter a conexão viva, reconectar e mostrar ao usuário se o Voicemod está conectado ou não.
3. **Lista de vozes** — listar as vozes disponíveis com o **id** de cada uma, sem depender de nomes digitados pelo usuário.
4. **Recompensa ligada por id** — o painel de recompensa escolhe a voz pelo id obtido na fase anterior; nada de escrever id à mão.
5. **Duração e recompensas simultâneas** — definir quanto tempo a voz vale e o que acontece quando duas recompensas são resgatadas ao mesmo tempo (uma espera, a última vence, etc.).
6. **Restauração de estado** — voltar ao microfone anterior ao fim da duração ou do fluxo, mesmo sem recompensa envolvida.
7. **Ausência do Voicemod** — quando não houver conexão, o fluxo continua normalmente sem erros no chat; a falha só aparece onde faz sentido (histórico ou aviso).
8. **Conclusão e cancelamento** — encerrar a troca de voz quando o tempo acabar ou o usuário cancelar a recompensa na Twitch, respeitando as permissões de reembolso (redeem/cancel) da conta do canal.
9. **Microfone virtual no OBS** — rotear o microfone virtual para uma fonte do OBS e registrar passo a passo na documentação, testado no aplicativo real.

Cuidados importantes desta integração:

- **Provar antes de prometer** — nada de prometer a funcionalidade antes de validar a API com o Voicemod aberto; as fases acima são ordem de execução, não garantia.
- **Não confundir com troca de TTS** — isto é sobre o microfone virtual usado como fonte de áudio, não sobre trocar o motor de voz do bot.
- **Licença do Voicemod** — o que está disponível pode depender da licença do usuário; não assumir conteúdo de licença sem verificar.
