export type VariableChoice={key:string;label:string;group:string;example?:string};
export const variableCatalog:VariableChoice[]=[
 {key:'commandCount',label:'Contagem do comando',group:'Execução',example:'12 — contador individual salvo'},
 {key:'local.aiResponse',label:'Resposta da IA',group:'IA',example:'Disponível depois de gerar a resposta'},
 {key:'local.aiSuccess',label:'IA respondeu com sucesso?',group:'IA',example:'false quando usou a alternativa ou uma prévia'},
 {key:'user',label:'Nome da pessoa',group:'Pessoa',example:'Ana'},
 {key:'userId',label:'Identificador da pessoa',group:'Pessoa'},
 {key:'randomViewer',label:'Nome sorteado no chat',group:'Pessoa',example:'Ana — cada ocorrência sorteia de novo'},
 {key:'random:1,50',label:'Número sorteado',group:'Execução',example:'17'},
 {key:'role',label:'Papel no chat',group:'Pessoa',example:'subscriber'},
 {key:'isModerator',label:'É moderador ou streamer?',group:'Pessoa'},
 {key:'isBroadcaster',label:'É o streamer?',group:'Pessoa'},
 {key:'isSubscriber',label:'Papel informado é assinante?',group:'Pessoa'},
 {key:'rawInput',label:'Texto depois do comando',group:'Mensagem',example:'quero jogar'},
 {key:'message',label:'Mensagem completa',group:'Mensagem',example:'!pedido quero jogar'},
 {key:'command',label:'Comando usado',group:'Mensagem',example:'!pedido'},
 {key:'arg0',label:'Primeira palavra do pedido',group:'Mensagem',example:'quero'},
 {key:'arg1',label:'Segunda palavra do pedido',group:'Mensagem',example:'jogar'},
 {key:'argCount',label:'Quantidade de palavras do pedido',group:'Mensagem',example:'2'},
 {key:'args',label:'Lista de palavras do pedido',group:'Mensagem'},
 {key:'channel',label:'Nome do canal',group:'Canal',example:'cactus'},
 {key:'channelId',label:'Identificador do canal',group:'Canal'},
 {key:'followerCount',label:'Seguidores do canal',group:'Canal',example:'1022 — atualiza a cada alerta'},
 {key:'subCount',label:'Assinantes do canal',group:'Canal',example:'42 — atualiza a cada alerta'},
 {key:'platform',label:'Plataforma',group:'Canal',example:'twitch'},
 {key:'profileName',label:'Nome do perfil',group:'Canal'},
 {key:'profileId',label:'Identificador do perfil',group:'Canal'},
 {key:'date',label:'Data de hoje',group:'Data e hora',example:'2026-09-23'},
 {key:'time',label:'Horário',group:'Data e hora',example:'19:30:00'},
 {key:'unixtime',label:'Instante Unix',group:'Data e hora'},
 {key:'eventType',label:'Tipo do evento',group:'Execução'},
 {key:'eventId',label:'Identificador do evento',group:'Execução'},
 {key:'subTier',label:'Nível do sub',group:'Eventos',example:'1'},
 {key:'subMonths',label:'Meses acumulados de sub',group:'Eventos'},
 {key:'subStreak',label:'Meses em sequência',group:'Eventos'},
 {key:'subMessage',label:'Mensagem do resub',group:'Eventos'},
 {key:'isGift',label:'É sub presenteado?',group:'Eventos'},
 {key:'gifterName',label:'Quem presenteou',group:'Eventos',example:'Anônimo quando oculto'},
 {key:'giftTotal',label:'Subs presenteados de uma vez',group:'Eventos'},
 {key:'giftTier',label:'Nível do presente',group:'Eventos'},
 {key:'raidViewers',label:'Pessoas trazidas pela raid',group:'Eventos'},
 {key:'raiderLogin',label:'Canal que fez a raid',group:'Eventos'},
 {key:'actionName',label:'Nome da automação',group:'Execução'},
 {key:'actionId',label:'Identificador da automação',group:'Execução'},
 {key:'simulated',label:'É uma simulação?',group:'Execução'},
 {key:'lf',label:'Quebra de linha',group:'Mensagem'},
 {key:'lastSpeech',label:'Última fala no microfone',group:'Voz',example:'bora de ranked'},
 {key:'liveSpeech',label:'Falas da escuta nesta sessão',group:'Voz',example:'bora de ranked | vamos ganhar hoje'},
 {key:'local.twitchGame',label:'Jogo resolvido pela ação Twitch',group:'Twitch',example:'VALORANT — vale nas ações seguintes'},
 {key:'local.twitchGameId',label:'ID do jogo resolvido',group:'Twitch'},
 {key:'local.twitchTitle',label:'Título aplicado pela ação Twitch',group:'Twitch'},
 {key:'local.twitchTarget',label:'Alvo da ação Twitch',group:'Twitch',example:'maria'},
];
export const scopeNames:Record<string,string>={local:'Só nesta execução',global:'Salva no perfil',user:'Salva por pessoa',session:'Perfil, até fechar o app',sessionUser:'Pessoa, até fechar o app',data:'Dados do evento'};
// `random:min,max` traz a faixa dentro do próprio código, por isso não é um nome de variável.
const drawKey=/^random:\s*-?\d{1,10}(\.\d{1,4})?\s*,\s*-?\d{1,10}(\.\d{1,4})?\s*$/;
export function variableLabel(key:string){
 if(key==='userName')return 'Nome da pessoa';
 if(key.startsWith('random:'))return 'Número sorteado';
 const builtin=variableCatalog.find(v=>v.key===key);if(builtin)return builtin.label;
 const [scope,...path]=key.split('.');return scopeNames[scope]?path.join('.')+' · '+scopeNames[scope]:key;
}
export function insertVariable(text:string,token:string,start:number,end:number){
 const a=Math.max(0,Math.min(start,text.length)),b=Math.max(a,Math.min(end,text.length));
 return {text:text.slice(0,a)+token+text.slice(b),caret:a+token.length};
}
export function makeVariableToken(key:string,fallback:string,format:string){
 if(!(drawKey.test(key)||/^[A-Za-z_][A-Za-z0-9_.]*$/.test(key)))throw Error('Escolha uma variável válida.');
 if(/[|{}]/.test(fallback))throw Error('O texto alternativo não pode conter barras verticais ou chaves.');
 if(!['','upper','lower','trim','number:0','number:2','length'].includes(format))throw Error('Formato inválido.');
 return '{{'+key+(fallback?'|default:'+fallback:'')+(format?'|'+format:'')+'}}';
}
/**
 * Variáveis locais usadas no texto que nenhuma ação deste fluxo define.
 * Sem isto a ação para no Histórico com "Variável ausente" e a mensagem não sai,
 * que era o caso de um timer citando {{local.aiResponse}} sem nenhuma ação de IA.
 * Um texto alternativo no próprio código (|default:) resolve e tira o aviso.
 */
export function missingLocals(text:string,actions:{kind:string;target:string;twOp?:string}[]):string[]{
 const defined=new Set<string>();
 for(const a of actions){
  if(a.kind==='ai'||a.kind==='ai.generate'){defined.add('local.aiResponse');defined.add('local.aiSuccess')}
  if(a.kind==='twitch'){
   if(a.twOp==='game'){defined.add('local.twitchGame');defined.add('local.twitchGameId')}
   else if(a.twOp==='title'){defined.add('local.twitchTitle')}
   else if(['timeout','ban','unban','warn','vip','unvip','shoutout','mention'].includes(a.twOp||'')){defined.add('local.twitchTarget')}
   else{defined.add('local.twitchGame');defined.add('local.twitchGameId');defined.add('local.twitchTitle');defined.add('local.twitchTarget')}
  }
  if(a.kind==='ai.generate'&&a.target.startsWith('local.'))defined.add(a.target);
  if((a.kind==='variable.set'||a.kind==='variable.increment')&&a.target.startsWith('local.'))defined.add(a.target);
 }
 const used=new Set<string>();
 for(const m of text.matchAll(/\{\{([^{}]+)\}\}/g)){
  const parts=m[1].split('|');const key=(parts[0]||'').trim();
  if(!key.startsWith('local.'))continue;
  if(parts.slice(1).some(p=>p.startsWith('default:')))continue;
  used.add(key);
 }
 return [...used].filter(k=>!defined.has(k)).sort();
}
export function messageParts(text:string):{text:string;label?:string}[]{
 // Display only: this never evaluates templates or any user-provided value.
 const pattern=/\\(?:\{\{|[$%])|\{\{([^{}]+)\}\}|%([A-Za-z_][A-Za-z0-9_]*)%/g;
 const parts:{text:string;label?:string}[]=[];let last=0;
 for(const m of text.matchAll(pattern)){
  const at=m.index!;if(at>last)parts.push({text:text.slice(last,at)});
  if(m[0].startsWith('\\')||(m[2]&&!variableCatalog.some(v=>v.key===m[2])&&m[2]!=='userName'))parts.push({text:m[0]});
  else {const key=(m[1]||m[2]).split('|')[0].trim();parts.push({text:m[0],label:variableLabel(key)});}
  last=at+m[0].length;
 }
 if(last<text.length)parts.push({text:text.slice(last)});return parts;
}
