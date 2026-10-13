# Manutenção do BotLive

- Todo commit precisa de uma versão superior à anterior e de notas detalhadas em `updates/<versão>.md`.
- Use `npm run update:prepare -- X.Y.Z caminho/das-notas.md`; não altere versões isoladamente.
- As notas devem explicar o que mudou, como usar, validação realmente executada e limitações. Atualize também o capítulo de uso afetado.
- Execute `npm run repo:setup` após clonar. O hook verifica o conteúdo staged; a CI verifica cada commit recebido.
- Nunca versione `.private`, `.tools`, dados de usuários, credenciais, instaladores ou artefatos de teste. Binários são publicados em Releases.

## Regras de tela (qualidade visual)

- Nunca esprema texto em coluna estreita: `.form-grid` usa colunas que só aparecem quando há largura para cada uma (mínimo de 240px) e dentro do inspetor de fluxos é sempre uma coluna.
- Rodapé de formulário (`.form-footer`) com texto de apoio em largura própria; o texto nunca divide a linha com botões nem causa rolagem horizontal.
- Ao mexer em tela, valide no navegador (`npm run dev`) com janela estreita (~1100px) e larga; screenshot é parte da entrega quando o assunto é layout.
- Toda tela com prévia + ajustes (Estúdio) segue o padrão `.lp-studio`: coluna da prévia à esquerda com `position: sticky` (a prévia acompanha a rolagem dos ajustes) e os controles em cards à direita; abaixo de 960px vira coluna única com a prévia em cima. Meta, Alertas e Chat no OBS já seguem o padrão — não crie variações, e cubra a tela nova com as capturas de 1100px e larga.
- Um push em `main` inicia validação e publicação Windows assinada. Não reutilize uma versão já publicada nem reescreva uma release existente.
- Preserve a chave de assinatura atual. Nunca imprima, copie para o código ou faça commit da chave privada e sua senha.
- Antes do push, execute os testes pertinentes e `npm run update:check`. Registre falhas e limitações sem afirmar uma homologação que não ocorreu.
