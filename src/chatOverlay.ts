/**
 * Motor do chat no overlay — compartilhado entre a fonte do OBS
 * (src/overlayApp.ts) e a prévia do Estúdio de chat (src/ChatObs.tsx).
 * O backend só transmite a mensagem reduzida (papel, badges, emotes); os
 * filtros, o layout da coluna e as animações saem da configuração salva no
 * perfil, então o que o streamer testa no Estúdio é o mesmo código que o OBS
 * roda. A estrutura segue o widget de chat de referência:
 * [listra] [badges] [nome colorido] [separador :] [texto com emotes].
 */
import {ensureFont} from './overlayDesign';

export type ChatFrag={t:'t'|'e';v:string;id?:string};
export type ChatMsg={
 id:string;user:string;userId:string;role:string;text:string;
 badges:string[];frags:ChatFrag[];first:boolean;bot:boolean;
};

export type BadgeKey='broadcaster'|'moderator'|'vip'|'subscriber'|'founder'|'prime'|'bits'|'gift'|'other';

export type ChatConfig={
 enabled:boolean;
 botName:string;showBot:boolean;hideBots:boolean;bots:string;
 hideCommands:boolean;blocked:string;
 direction:'up'|'down';max:number;hideAfter:number;enter:'fade'|'slide'|'pixel'|'type';
 stripes:boolean;align:'left'|'center'|'right';width:number;
 upper:boolean;nameFixed:string;color:string;
 font:string;fontSize:number;nameSize:number;
 badges:Record<BadgeKey,boolean>;
 hlMode:'none'|'bar'|'bg'|'both';
 hl:{broadcaster:string;moderator:string;vip:string;subscriber:string};
 firstOn:boolean;firstColor:string;
};

/** Padrão espelhado em chat_overlay.rs (defaults). */
export const DEFAULT_CHAT:ChatConfig={
 enabled:false,botName:'BotLive',showBot:true,hideBots:false,bots:'',
 hideCommands:false,blocked:'',
 direction:'up',max:100,hideAfter:0,enter:'fade',
 stripes:false,align:'left',width:360,
 upper:false,nameFixed:'',color:'#ffffff',
 font:'Segoe UI',fontSize:16,nameSize:16,
 badges:{broadcaster:true,moderator:true,vip:true,subscriber:true,founder:true,prime:true,bits:true,gift:true,other:true},
 hlMode:'bar',
 hl:{broadcaster:'#e91916',moderator:'#00ad03',vip:'#e005b9',subscriber:'#1f9bff'},
 firstOn:true,firstColor:'#ffb26b',
};

const clamp=(v:unknown,min:number,max:number,fallback:number):number=>{
 const n=typeof v==='number'?v:Number(v);
 return Number.isFinite(n)?Math.min(max,Math.max(min,n)):fallback;
};
const str=(v:unknown,max:number,fallback:string):string=>(typeof v==='string'&&v.length<=max?v:fallback);
const bool=(v:unknown,fallback:boolean):boolean=>(typeof v==='boolean'?v:fallback);
const pick=<T extends string>(v:unknown,list:readonly T[],fallback:T):T=>(typeof v==='string'&&list.includes(v as T)?v as T:fallback);
const color=(v:unknown,fallback:string):string=>(typeof v==='string'&&/^#[0-9a-fA-F]{3,8}$/.test(v)?v:fallback);

const FONT_SET=new Set([
 'Segoe UI','Arial','Georgia','Verdana','Trebuchet MS','Courier New',
 'Inter','Montserrat','Poppins','Rubik','Orbitron','Bebas Neue',
 'Press Start 2P','Playfair Display','Rajdhani','Quicksand',
]);
const BADGE_KEYS:BadgeKey[]=['broadcaster','moderator','vip','subscriber','founder','prime','bits','gift','other'];

/** Aceita o JSON salvo (ou nada) e devolve uma configuração sempre utilizável. */
export function normalizeChat(v:unknown):ChatConfig{
 const o=(v&&typeof v==='object'?v:{}) as Record<string,unknown>;
 const rawBadges=(o.badges&&typeof o.badges==='object'?o.badges:{}) as Record<string,unknown>;
 const badges={} as Record<BadgeKey,boolean>;
 for(const k of BADGE_KEYS)badges[k]=bool(rawBadges[k],DEFAULT_CHAT.badges[k]);
 const rawHl=(o.hl&&typeof o.hl==='object'?o.hl:{}) as Record<string,unknown>;
 const font=typeof o.font==='string'&&FONT_SET.has(o.font)?o.font:DEFAULT_CHAT.font;
 return{
  enabled:bool(o.enabled,DEFAULT_CHAT.enabled),
  botName:str(o.botName,64,DEFAULT_CHAT.botName),
  showBot:bool(o.showBot,DEFAULT_CHAT.showBot),
  hideBots:bool(o.hideBots,DEFAULT_CHAT.hideBots),
  bots:str(o.bots,200,DEFAULT_CHAT.bots),
  hideCommands:bool(o.hideCommands,DEFAULT_CHAT.hideCommands),
  blocked:str(o.blocked,200,DEFAULT_CHAT.blocked),
  direction:pick(o.direction,['up','down'] as const,DEFAULT_CHAT.direction),
  max:clamp(o.max,10,300,DEFAULT_CHAT.max),
  hideAfter:clamp(o.hideAfter,0,120,DEFAULT_CHAT.hideAfter),
  enter:pick(o.enter,['fade','slide','pixel','type'] as const,DEFAULT_CHAT.enter),
  stripes:bool(o.stripes,DEFAULT_CHAT.stripes),
  align:pick(o.align,['left','center','right'] as const,DEFAULT_CHAT.align),
  width:clamp(o.width,240,960,DEFAULT_CHAT.width),
  upper:bool(o.upper,DEFAULT_CHAT.upper),
  nameFixed:typeof o.nameFixed==='string'&&(/^#[0-9a-fA-F]{3,8}$/.test(o.nameFixed)||o.nameFixed==='')?o.nameFixed:DEFAULT_CHAT.nameFixed,
  color:color(o.color,DEFAULT_CHAT.color),
  font,fontSize:clamp(o.fontSize,10,28,DEFAULT_CHAT.fontSize),nameSize:clamp(o.nameSize,10,36,DEFAULT_CHAT.nameSize),
  badges,
  hlMode:pick(o.hlMode,['none','bar','bg','both'] as const,DEFAULT_CHAT.hlMode),
  hl:{
   broadcaster:color(rawHl.broadcaster,DEFAULT_CHAT.hl.broadcaster),
   moderator:color(rawHl.moderator,DEFAULT_CHAT.hl.moderator),
   vip:color(rawHl.vip,DEFAULT_CHAT.hl.vip),
   subscriber:color(rawHl.subscriber,DEFAULT_CHAT.hl.subscriber),
  },
  firstOn:bool(o.firstOn,DEFAULT_CHAT.firstOn),
  firstColor:color(o.firstColor,DEFAULT_CHAT.firstColor),
 };
}

// A Twitch não expõe a cor do nome pelo EventSub; usamos uma cor estável por
// espectador, tirada do id, da própria paleta de cores da Twitch.
const TWITCH_COLORS=['#e91916','#e4690f','#b98500','#00b509','#0f9c9c','#1f70c7','#5c16c5','#a02886'];
export function userColor(userId:string):string{
 let h=5381;
 for(let i=0;i<userId.length;i++)h=((h<<5)+h+userId.charCodeAt(i))>>>0;
 return TWITCH_COLORS[h%TWITCH_COLORS.length];
}

const csv=(v:string):string[]=>v.split(',').map(s=>s.trim().toLowerCase()).filter(Boolean);

/** Nome tratado como bot: lista configurada ou sufixo "bot". */
export function isBotName(user:string,cfg:ChatConfig):boolean{
 const n=user.trim().toLowerCase();
 return csv(cfg.bots).includes(n)||n.endsWith('bot');
}

/** Filtros do Estúdio e do OBS — os mesmos dois lugares. */
export function chatVisible(msg:ChatMsg,cfg:ChatConfig):boolean{
 if(msg.bot&&!cfg.showBot)return false;
 if(!msg.bot&&cfg.hideBots&&isBotName(msg.user,cfg))return false;
 if(cfg.hideCommands&&msg.text.trimStart().startsWith('!'))return false;
 const blocked=csv(cfg.blocked);
 if(blocked.length){
  const low=msg.text.toLowerCase();
  if(blocked.some(w=>low.includes(w)))return false;
 }
 return true;
}

// Badges desenhadas pelo próprio motor (sem depender da API de badges da
// Twitch): set conhecido vira ícone, o resto entra como "outros".
const BADGE_SET:Record<string,BadgeKey>={
 broadcaster:'broadcaster',moderator:'moderator',vip:'vip',subscriber:'subscriber',
 founder:'founder',premium:'prime',prime:'prime',bits:'bits','sub-gifter':'gift',gift:'gift',
};
const BADGE_ART:Record<BadgeKey,[string,string]>={
 broadcaster:['#e91916','<path d="M2 4.5h7.5v7H2zM10 7l4-2.5v7L10 9z"/>'],
 moderator:['#00ad03','<path d="M8 1l6 2v4.6c0 3.5-2.6 5.9-6 6.9-3.4-1-6-3.4-6-6.9V3z"/>'],
 vip:['#e005b9','<path d="M8 14.2S1.2 10 1.2 5.9C1.2 3.5 3 2 5 2c1.3 0 2.3.7 3 1.7C8.7 2.7 9.7 2 11 2c2 0 3.8 1.5 3.8 3.9C14.8 10 8 14.2 8 14.2z"/>'],
 subscriber:['#9147ff','<path d="M8 1l2 4.3 4.7.5-3.5 3.2 1 4.6L8 11.3 3.8 13.6l1-4.6L1.3 5.8 6 5.3z"/>'],
 founder:['#ad7542','<path d="M8 1l2 4.3 4.7.5-3.5 3.2 1 4.6L8 11.3 3.8 13.6l1-4.6L1.3 5.8 6 5.3z"/>'],
 prime:['#00a3ff','<path d="M1 4.5l3 2.5L8 2l4 5 3-2.5V13H1zM1 13h14v1.6H1z"/>'],
 bits:['#9147ff','<path d="M8 1l6 5.5L8 15 2 6.5z"/>'],
 gift:['#ff5d89','<path d="M2 6.5h12V14H2zM1 3h14v3H1zM8 3v11"/>'],
 other:['#9aa0a6','<circle cx="8" cy="8" r="5.5"/>'],
};

let styleOn=false;
/** Estilo do chat injetado uma vez — a fonte do OBS e o Estúdio usam o mesmo. */
export function ensureChatStyle():void{
 if(styleOn||typeof document==='undefined')return;
 styleOn=true;
 const css=`
.bl-chat-col{position:absolute;display:flex;flex-direction:column;justify-content:flex-end;gap:5px;font-family:var(--bl-chat-font,"Segoe UI",sans-serif);}
.bl-chat-col.bl-chat-fixed{position:fixed;z-index:40}
.bl-chat-col.down{justify-content:flex-start}
.bl-chat-msg{position:relative;display:flex;flex-wrap:wrap;align-items:baseline;column-gap:6px;row-gap:2px;padding:5px 9px;box-sizing:border-box;line-height:1.35;}
.bl-chat-stripes .bl-chat-msg:nth-child(odd){background:rgba(255,255,255,.055)}
.bl-chat-tint{position:absolute;inset:0;opacity:0;pointer-events:none}
.bl-chat-msg.hl-bg .bl-chat-tint{background:var(--hl,#000);opacity:.16}
.bl-chat-msg.hl-bar{border-left:3px solid var(--hl,transparent)}
.bl-chat-name{font-weight:700}
.bl-chat-sep{opacity:.7;margin-left:-3px}
.bl-chat-badge{display:inline-flex;align-items:center;align-self:center}
.bl-chat-badge svg{width:100%;height:100%;display:block}
.bl-chat-text{flex:1 1 auto;min-width:0;word-break:break-word}
.bl-chat-emote{vertical-align:-.25em;margin:0 1px}
@keyframes bl-chat-fade{from{opacity:0}to{opacity:1}}
@keyframes bl-chat-slide{from{opacity:0;transform:translateY(14px)}to{opacity:1;transform:none}}
@keyframes bl-chat-pixel{from{opacity:0}50%{opacity:.5}to{opacity:1}}
@keyframes bl-chat-type{from{clip-path:inset(0 100% 0 0)}to{clip-path:inset(0 0 0 0)}}
.bl-enter-fade{animation:bl-chat-fade .18s ease both}
.bl-enter-slide{animation:bl-chat-slide .22s ease both}
.bl-enter-pixel{animation:bl-chat-pixel .3s steps(3,end) both}
.bl-enter-type{animation:bl-chat-type .35s steps(10,end) both}
.bl-leaving{opacity:0 !important;transition:opacity .3s ease}`;
 const el=document.createElement('style');
 el.id='bl-chat-style';
 el.textContent=css;
 document.head.appendChild(el);
}

/** Posiciona a coluna dentro da cena (overlay) ou da prévia (Estúdio). */
export function layoutChat(host:HTMLElement,cfg:ChatConfig,inset=24):void{
 ensureChatStyle();
 ensureFont(cfg.font);
 host.style.width=cfg.width+'px';
 host.style.left=cfg.align==='left'?inset+'px':cfg.align==='right'?'auto':'50%';
 host.style.right=cfg.align==='right'?inset+'px':'auto';
 host.style.transform=cfg.align==='center'?'translateX(-50%)':'none';
 host.style.bottom=inset+'px';
 host.style.maxHeight=`calc(100% - ${inset*2}px)`;
 host.style.setProperty('--bl-chat-font',`"${cfg.font}", "Segoe UI", sans-serif`);
 host.style.color=cfg.color;
 host.classList.toggle('down',cfg.direction==='down');
 host.classList.toggle('bl-chat-stripes',cfg.stripes);
}

function highlightOf(msg:ChatMsg,cfg:ChatConfig):string{
 if(msg.role==='broadcaster')return cfg.hl.broadcaster;
 if(msg.role==='moderator')return cfg.hl.moderator;
 if(msg.role==='vip')return cfg.hl.vip;
 if(msg.role==='subscriber')return cfg.hl.subscriber;
 if(msg.first&&cfg.firstOn)return cfg.firstColor;
 return '';
}

function buildMessage(msg:ChatMsg,cfg:ChatConfig):HTMLElement{
 const el=document.createElement('div');
 el.className='bl-chat-msg';
 el.setAttribute('data-id',msg.id);
 el.setAttribute('data-user',msg.userId);
 const tint=document.createElement('span');
 tint.className='bl-chat-tint';
 el.appendChild(tint);
 const badgePx=Math.max(11,Math.round(cfg.nameSize*1.05));
 const seen=new Set<BadgeKey>();
 for(const raw of msg.badges){
  const key=BADGE_SET[raw]||'other';
  if(!cfg.badges[key]||seen.has(key))continue;
  seen.add(key);
  const [bcolor,art]=BADGE_ART[key];
  const b=document.createElement('span');
  b.className='bl-chat-badge';
  b.style.color=bcolor;
  b.style.width=b.style.height=badgePx+'px';
  b.innerHTML='<svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">'+art+'</svg>';
  el.appendChild(b);
 }
 const name=document.createElement('span');
 name.className='bl-chat-name';
 name.textContent=cfg.upper?msg.user.toUpperCase():msg.user;
 name.style.color=cfg.nameFixed||userColor(msg.userId);
 name.style.fontSize=cfg.nameSize+'px';
 el.appendChild(name);
 const hlColor=highlightOf(msg,cfg);
 if(hlColor&&cfg.hlMode!=='none'){
  el.style.setProperty('--hl',hlColor);
  if(cfg.hlMode==='bar'||cfg.hlMode==='both')el.classList.add('hl-bar');
  if(cfg.hlMode==='bg'||cfg.hlMode==='both')el.classList.add('hl-bg');
 }
 const sep=document.createElement('span');
 sep.className='bl-chat-sep';
 sep.textContent=':';
 sep.style.fontSize=cfg.fontSize+'px';
 el.appendChild(sep);
 const text=document.createElement('span');
 text.className='bl-chat-text';
 text.style.fontSize=cfg.fontSize+'px';
 const onlyEmotes=msg.frags.length>0&&msg.frags.every(f=>f.t==='e');
 const emotePx=Math.round(cfg.fontSize*(onlyEmotes?2.4:1.4));
 for(const f of msg.frags){
  if(f.t==='e'&&f.id){
   const img=document.createElement('img');
   img.className='bl-chat-emote';
   img.src=`https://static-cdn.jtvnw.net/emoticons/v2/${encodeURIComponent(f.id)}/default/dark/2.0`;
   img.alt=f.v;img.title=f.v;img.style.height=emotePx+'px';
   // Sem internet o emote vira o próprio texto, nunca um ícone quebrado.
   img.onerror=()=>{const s=document.createElement('span');s.textContent=f.v;img.replaceWith(s)};
   text.appendChild(img);
  }else{
   text.appendChild(document.createTextNode(f.v));
  }
 }
 el.appendChild(text);
 return el;
}

type Timed=HTMLElement&{_blTimer?:number};
const clearTimer=(node:Element)=>{const t=(node as Element&{_blTimer?:number})._blTimer;if(t){window.clearTimeout(t);(node as Element&{_blTimer?:number})._blTimer=0}};
const leave=(node:Element)=>{clearTimer(node);node.classList.add('bl-leaving');window.setTimeout(()=>node.remove(),320)};

/** Empurra a mensagem na coluna, respeitando direção, teto e some-apois. */
export function pushChat(host:HTMLElement,msg:ChatMsg,cfg:ChatConfig,inset=24):void{
 layoutChat(host,cfg,inset);
 const node=buildMessage(msg,cfg) as Timed;
 node.classList.add('bl-enter-'+cfg.enter);
 if(cfg.direction==='down')host.prepend(node);else host.appendChild(node);
 while(host.childElementCount>cfg.max){
  const old=cfg.direction==='down'?host.lastElementChild:host.firstElementChild;
  if(!old)break;
  clearTimer(old);old.remove();
 }
 if(cfg.hideAfter>0)node._blTimer=window.setTimeout(()=>leave(node),cfg.hideAfter*1000);
}

const byAttr=(host:HTMLElement,name:string,value:string):Element|undefined=>
 Array.from(host.children).find(c=>c.getAttribute(name)===value);

/** Mensagem apagada pela moderação some da coluna. */
export function deleteChat(host:HTMLElement,id:string):void{
 const found=byAttr(host,'data-id',id);
 if(found){clearTimer(found);found.remove()}
}

/** Mensagens de um espectador (limpar por usuário) somem juntas. */
export function deleteUserChat(host:HTMLElement,userId:string):void{
 Array.from(host.children).filter(c=>c.getAttribute('data-user')===userId).forEach(c=>{clearTimer(c);c.remove()});
}

/** Chat limpo pela Twitch: coluna zerada. */
export function clearChat(host:HTMLElement):void{
 Array.from(host.children).forEach(clearTimer);
 host.replaceChildren();
}

// Mensagens de teste do Estúdio: entram pelo mesmo motor das reais.
export const CHAT_SAMPLES=['common','mod','vip','sub','first','emotes','emotesOnly','bot'] as const;
export type ChatSample=(typeof CHAT_SAMPLES)[number];

export function chatSample(kind:ChatSample):ChatMsg{
 const base=(p:Partial<ChatMsg>):ChatMsg=>({
  id:'test-'+kind+'-'+Math.random().toString(36).slice(2,8),
  user:'Ana',userId:'ana',role:'everyone',
  text:'Boa noite, gente! A live hoje está show.',
  badges:[],frags:[{t:'t',v:'Boa noite, gente! A live hoje está show.'}],
  first:false,bot:false,...p,
 });
 switch(kind){
  case 'mod':return base({user:'ModeradorAna',userId:'mod-ana',role:'moderator',badges:['moderator','subscriber'],text:'Calma, gente, regras do chat à vista!'});

  case 'vip':return base({user:'VipBia',userId:'vip-bia',role:'vip',badges:['vip'],text:'Cheguei! Alguém me chamou?'});
  case 'sub':return base({user:'SubLeo',userId:'sub-leo',role:'subscriber',badges:['subscriber','founder'],text:'Apoiando desde o início, valeu demais!'});

  case 'first':return base({user:'Novato',userId:'novo-1',first:true,text:'Primeira vez por aqui, salve!'});

  case 'emotes':return base({user:'Emoteiro',userId:'emote-1',text:'olha o Kappa que achei',frags:[{t:'t',v:'olha o '},{t:'e',v:'Kappa',id:'25'},{t:'t',v:' que achei'}]});
  case 'emotesOnly':return base({user:'Emoteiro',userId:'emote-2',text:'Kappa Kappa',frags:[{t:'e',v:'Kappa',id:'25'},{t:'e',v:'Kappa',id:'25'}]});
  case 'bot':return base({user:DEFAULT_CHAT.botName,userId:'bot',bot:true,text:'Boa noite, Ana! Comando registrado.'});
  default:return base({});
 }
}
