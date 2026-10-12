import {describe,expect,it} from 'vitest';
import {
 CHAT_SAMPLES,DEFAULT_CHAT,chatSample,chatVisible,isBotName,normalizeChat,userColor,
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
