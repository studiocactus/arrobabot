export type VariableChoice={key:string;label:string;group:string;example?:string};
export const variableCatalog:VariableChoice[]=[
 {key:'commandCount',label:'Contagem do comando',group:'Execução',example:'12 — contador individual salvo'},
 {key:'local.aiResponse',label:'Resposta da IA',group:'IA',example:'Disponível depois de gerar a resposta'},
 {key:'local.aiSuccess',label:'IA respondeu com sucesso?',group:'IA',example:'false quando usou a alternativa ou uma prévia'},
 {key:'user',label:'Nome da pessoa',group:'Pessoa',example:'Ana'},
 {key:'userId',label:'Identificador da pessoa',group:'Pessoa'},
 {key:'randomViewer',label:'Nome sorteado no chat',group:'Pessoa',example:'Ana'},
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
 {key:'platform',label:'Plataforma',group:'Canal',example:'twitch'},
 {key:'profileName',label:'Nome do perfil',group:'Canal'},
 {key:'profileId',label:'Identificador do perfil',group:'Canal'},
 {key:'date',label:'Data de hoje',group:'Data e hora',example:'2026-09-23'},
 {key:'time',label:'Horário',group:'Data e hora',example:'19:30:00'},
 {key:'unixtime',label:'Instante Unix',group:'Data e hora'},
 {key:'eventType',label:'Tipo do evento',group:'Execução'},
 {key:'eventId',label:'Identificador do evento',group:'Execução'},
 {key:'actionName',label:'Nome da automação',group:'Execução'},
 {key:'actionId',label:'Identificador da automação',group:'Execução'},
 {key:'simulated',label:'É uma simulação?',group:'Execução'},
 {key:'lf',label:'Quebra de linha',group:'Mensagem'},
];
export const scopeNames:Record<string,string>={local:'Só nesta execução',global:'Salva no perfil',user:'Salva por pessoa',session:'Perfil, até fechar o app',sessionUser:'Pessoa, até fechar o app',data:'Dados do evento'};
export function variableLabel(key:string){
 if(key==='userName')return 'Nome da pessoa';
 const builtin=variableCatalog.find(v=>v.key===key);if(builtin)return builtin.label;
 const [scope,...path]=key.split('.');return scopeNames[scope]?path.join('.')+' · '+scopeNames[scope]:key;
}
export function insertVariable(text:string,token:string,start:number,end:number){
 const a=Math.max(0,Math.min(start,text.length)),b=Math.max(a,Math.min(end,text.length));
 return {text:text.slice(0,a)+token+text.slice(b),caret:a+token.length};
}
export function makeVariableToken(key:string,fallback:string,format:string){
 if(!/^[A-Za-z_][A-Za-z0-9_.]*$/.test(key))throw Error('Escolha uma variável válida.');
 if(/[|{}]/.test(fallback))throw Error('O texto alternativo não pode conter barras verticais ou chaves.');
 if(!['','upper','lower','trim','number:0','number:2','length'].includes(format))throw Error('Formato inválido.');
 return '{{'+key+(fallback?'|default:'+fallback:'')+(format?'|'+format:'')+'}}';
}
export function messageParts(text:string):{text:string;label?:string}[]{
 // Display only: this never evaluates templates or any user-provided value.
 const pattern=/\\(?:\{\{|[$%])|\{\{([^{}]+)\}\}|\$([A-Za-z_][A-Za-z0-9_]*)|%([A-Za-z_][A-Za-z0-9_]*)%/g;
 const parts:{text:string;label?:string}[]=[];let last=0;
 for(const m of text.matchAll(pattern)){
  const at=m.index!;if(at>last)parts.push({text:text.slice(last,at)});
  if(m[0].startsWith('\\')||(m[2]&&!variableCatalog.some(v=>v.key===m[2])&&m[2]!=='userName'))parts.push({text:m[0]});
  else {const key=(m[1]||m[2]||m[3]).split('|')[0].trim();parts.push({text:m[0],label:variableLabel(key)});}
  last=at+m[0].length;
 }
 if(last<text.length)parts.push({text:text.slice(last)});return parts;
}
