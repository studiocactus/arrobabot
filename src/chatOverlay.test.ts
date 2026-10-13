import {describe,expect,it} from 'vitest';
import {
 CHAT_EVENT_KINDS,CHAT_SAMPLES,DEFAULT_CHAT,chatSample,chatVisible,eventBoxHtml,fillEventText,
 isBotName,isChatEventKind,nameColor,normalizeChat,userColor,
 type ChatMsg,
} from './chatOverlay';

const msg=(p:Partial<ChatMsg>):ChatMsg=>({
 id:'m1',user:'Ana',userId:'ana',role:'everyone',text:'oi tudo bem',
 badges:[],frags:[{t:'t',v:'oi tudo bem'}],first:false,bot:false,...p,
});

describe('normalizeChat',()=>{
 it('sem nada devolve o padrão desligado espelhado no backend',()=>{
  expect(normalizeChat(null)).toEqual(DEFAULT_CHAT);
  expect(normalizeChat(null).enabled).toBe(false);
 });
 it('limita faixas e descarta valores estranhos',()=>{
  const n=normalizeChat({max:999,hideAfter:500,width:10,fontSize:90,direction:'sideways',enter:'boom',font:'Comic Sans',color:'vermelho',nameFixed:'#f00'});
  expect(n.max).toBe(300);
  expect(n.hideAfter).toBe(120);
  expect(n.width).toBe(240);
  expect(n.fontSize).toBe(28);
  expect(n.direction).toBe('up');
  expect(n.enter).toBe('fade');
  expect(n.font).toBe(DEFAULT_CHAT.font);
  expect(n.color).toBe(DEFAULT_CHAT.color);
  expect(n.nameFixed).toBe('#f00');
 });
 it('badges e destaques parciais completam sem sumir com o resto',()=>{
  const n=normalizeChat({badges:{vip:false},hl:{moderator:'#123456'}});
  expect(n.badges.vip).toBe(false);
  expect(n.badges.moderator).toBe(true);
  expect(n.hl.moderator).toBe('#123456');
  expect(n.hl.broadcaster).toBe(DEFAULT_CHAT.hl.broadcaster);
 });
});

describe('userColor',()=>{
 it('é estável por espectador e sai da paleta da Twitch',()=>{
  const palette=new Set(['#e91916','#e4690f','#b98500','#00b509','#0f9c9c','#1f70c7','#5c16c5','#a02886']);
  const a=userColor('ana');
  expect(a).toBe(userColor('ana'));
  expect(palette.has(a)).toBe(true);
  expect(userColor('bia')).toMatch(/^#/);
 });
});

describe('chatVisible',()=>{
 it('esconde comandos, bots da lista, sufixo bot e palavras bloqueadas',()=>{
  expect(chatVisible(msg({text:'!discord'}),normalizeChat({hideCommands:true}))).toBe(false);
  expect(chatVisible(msg({text:'!discord'}),normalizeChat(null))).toBe(true);
  expect(chatVisible(msg({user:'NightBot'}),normalizeChat({hideBots:true}))).toBe(false);
  expect(chatVisible(msg({user:'meubot'}),normalizeChat({hideBots:true}))).toBe(false);
  expect(chatVisible(msg({user:'meubot'}),normalizeChat({hideBots:true,bots:'ana'}))).toBe(false);
  expect(chatVisible(msg({user:'Robo'}),normalizeChat({hideBots:true,bots:'robo'}))).toBe(false);
  expect(chatVisible(msg({text:'vamos de Ranked'}),normalizeChat({blocked:'rankeado, ranked'}))).toBe(false);
  expect(chatVisible(msg({text:'valeu!'}),normalizeChat({blocked:'rankeado, ranked'}))).toBe(true);
 });
 it('mensagem do bot obedece o interruptor próprio',()=>{
  expect(chatVisible(msg({bot:true,user:'BotLive'}),normalizeChat(null))).toBe(true);
  expect(chatVisible(msg({bot:true,user:'BotLive'}),normalizeChat({showBot:false}))).toBe(false);
 });
 it('esconder bots não pega as mensagens do próprio bot',()=>{
  expect(chatVisible(msg({bot:true,user:'BotLive'}),normalizeChat({hideBots:true}))).toBe(true);
 });
});

describe('isBotName',()=>{
 it('lista configurada, sufixo bot e nomes comuns que não são bots',()=>{
  const cfg=normalizeChat({bots:'streamelements, nightbot'});
  expect(isBotName('StreamElements',cfg)).toBe(true);
  expect(isBotName('nightbot',cfg)).toBe(true);
  expect(isBotName('meubot',cfg)).toBe(true);
  expect(isBotName('Ana',cfg)).toBe(false);
  expect(isBotName('Robson',cfg)).toBe(false);
 });
});

describe('chatSample',()=>{
 it('todos os exemplos passam pelos filtros padrão e carregam o papel',()=>{
  const cfg=normalizeChat(null);
  for(const kind of CHAT_SAMPLES){
   const m=chatSample(kind);
   expect(chatVisible(m,cfg),kind).toBe(true);
  }
  expect(chatSample('mod').badges).toContain('moderator');
  expect(chatSample('vip').role).toBe('vip');
  expect(chatSample('first').first).toBe(true);
  expect(chatSample('bot').bot).toBe(true);
  expect(chatSample('emotes').frags.some(f=>f.t==='e'&&f.id)).toBe(true);
  expect(chatSample('emotesOnly').frags.every(f=>f.t==='e')).toBe(true);
 });
 it('cada exemplo tem id próprio para a coluna não colapsar',()=>{
  const ids=new Set(CHAT_SAMPLES.map(k=>chatSample(k).id));
  expect(ids.size).toBe(CHAT_SAMPLES.length);
 });
});

describe('origem da cor do nome (3 vias)',()=>{
 const cfg=(nameMode:'twitch'|'custom'|'fixed',nameFixed=''):ReturnType<typeof normalizeChat>=>
  normalizeChat({nameMode,nameFixed});
 it('twitch usa a cor real da mensagem e cai no hash quando vazia',()=>{
  expect(nameColor(msg({color:'#9146ff'}),cfg('twitch'))).toBe('#9146ff');
  expect(nameColor(msg({color:''}),cfg('twitch'))).toBe(userColor('ana'));
  expect(nameColor(msg({color:'azul'}),cfg('twitch')),'fora de hex não passa').toBe(userColor('ana'));
 });
 it('personalizadas é sempre o hash do espectador, ignorando a cor real',()=>{
  expect(nameColor(msg({color:'#9146ff'}),cfg('custom'))).toBe(userColor('ana'));
 });
 it('fixa usa a cor do perfil e sem cor escolhida volta ao hash',()=>{
  expect(nameColor(msg({color:'#9146ff'}),cfg('fixed','#ff00aa'))).toBe('#ff00aa');
  expect(nameColor(msg({}),cfg('fixed',''))).toBe(userColor('ana'));
 });
 it('configuração antiga sem nameMode mantém o comportamento de antes',()=>{
  expect(normalizeChat({nameFixed:'#ff00aa'}).nameMode,'sem chave, o padrão novo vale').toBe('twitch');
 });
});

describe('prefixo antes do nome',()=>{
 it('normaliza texto, cor e interruptor com limites do backend',()=>{
  const n=normalizeChat({prefix:'>>',prefixColor:'nope',prefixOn:false});
  expect(n.prefix).toBe('>>');
  expect(n.prefixColor).toBe(DEFAULT_CHAT.prefixColor);
  expect(n.prefixOn).toBe(false);
  const long=normalizeChat({prefix:'x'.repeat(17)});
  expect(long.prefix,'acima de 16 caracteres cai no padrão').toBe(DEFAULT_CHAT.prefix);
 });
});

describe('eventos na coluna',()=>{
 it('normaliza os 7 tipos com interruptor, cor, som e texto',()=>{
  const n=normalizeChat({ev:{raid:{on:false,color:'#123456',sound:'ping.wav',text:'entrou {name}'}}});
  expect(Object.keys(n.ev).sort()).toEqual([...CHAT_EVENT_KINDS].sort());
  expect(n.ev.raid).toEqual({on:false,color:'#123456',sound:'ping.wav',text:'entrou {name}'});
  expect(n.ev.follow.on).toBe(true);
  expect(n.ev.follow.text).toBe(DEFAULT_CHAT.ev.follow.text);
  const bad=normalizeChat({ev:{follow:{color:'vermelho'}}});
  expect(bad.ev.follow.color,'cor inválida volta ao padrão').toBe(DEFAULT_CHAT.ev.follow.color);
  const long=normalizeChat({ev:{follow:{text:'x'.repeat(161)}}});
  expect(long.ev.follow.text).toBe(DEFAULT_CHAT.ev.follow.text);
 });
 it('desligado por padrão para não duplicar a camada de alertas',()=>{
  expect(normalizeChat(null).eventsOn).toBe(false);
  expect(normalizeChat(null).evShowMsg).toBe(true);
 });
 it('isChatEventKind só aceita os tipos do catálogo',()=>{
  expect(isChatEventKind('raid')).toBe(true);
  expect(isChatEventKind('tip')).toBe(false);
  expect(isChatEventKind('')).toBe(false);
 });
 it('fillEventText preenche marcadores e escapa valores de espectador',()=>{
  const out=fillEventText('{name} / {amount} BITS',{name:'<b>ana</b>',amount:500});
  expect(out).toBe('&lt;b&gt;ana&lt;/b&gt; / 500 BITS');
  expect(fillEventText('sem marcador',{name:'ana'})).toBe('sem marcador');
 });
 it('eventBoxHtml espelha o backend: texto e linha de mensagem escapada',()=>{
  const cfg=normalizeChat(null);
  expect(eventBoxHtml(cfg,'follow')).toContain('NOVO FOLLOW / Sortuda');
  const resub=eventBoxHtml(cfg,'resub');
  expect(resub).toContain('RESUB / Sortuda / 14 MESES / TIER 3');
  expect(resub).toContain('já são 14 meses de live!');
  const semMensagem=normalizeChat({evShowMsg:false});
  expect(eventBoxHtml(semMensagem,'resub')).not.toContain('bl-evmsg');
  const perigoso=normalizeChat({ev:{follow:{text:'{name}',color:'#c6ff00',sound:'',on:true}}});
  expect(eventBoxHtml(perigoso,'follow'),'placeholder escapa como o backend').toBe('Sortuda');
 });
});

describe('modo código',()=>{
 it('normaliza os três blocos com teto de 20 mil caracteres',()=>{
  const n=normalizeChat({code:{html:'<div>x</div>',css:'.a{}',js:'root.title=1'}});
  expect(n.code).toEqual({html:'<div>x</div>',css:'.a{}',js:'root.title=1'});
  expect(normalizeChat({code:{js:'x'.repeat(20_001)}}).code.js).toBe('');
  expect(normalizeChat({code:'nope'}).code).toEqual({html:'',css:'',js:''});
  expect(normalizeChat(null).code).toEqual({html:'',css:'',js:''});
 });
});
