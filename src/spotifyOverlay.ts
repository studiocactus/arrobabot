/**
 * Configuração e leitura da música tocando — compartilhadas entre a tela
 * Spotify, a prévia do Estúdio (Fase B: overlay) e o catálogo de variáveis.
 * O backend transmite a faixa reduzida; aqui só se normaliza o que veio do
 * banco ou do evento, sempre com padrão para o que faltar.
 */

export type SpotifyLayout='card'|'compact'|'stacked'|'minimal';
export type SpotifyCover='square'|'circle';
export type SpotifyEmpty='hide'|'message';
export type SpotifyTransition='fade'|'slide'|'none';
export type SpotifyPosition='bottom-left'|'bottom-right'|'top-left'|'top-right'|'center';

export type SpotifyDesign={
 layout:SpotifyLayout;cover:SpotifyCover;coverGlow:boolean;coverBlur:boolean;
 showArt:boolean;showAlbum:boolean;showProgress:boolean;showTimes:boolean;
 bg:string;titleColor:string;artistColor:string;albumColor:string;
 barBg:string;barColor:string;barHeight:number;barRadius:number;
 font:string;titleSize:number;artistSize:number;artSize:number;artRadius:number;
 width:number;padding:number;radius:number;
 empty:SpotifyEmpty;emptyText:string;emptyArtist:string;
 hideOnPause:boolean;pauseDelay:number;
 position:SpotifyPosition;
 transition:SpotifyTransition;exitTransition:SpotifyTransition;transitionMs:number;
};
export type SpotifyConfig={enabled:boolean;clientId:string;design:SpotifyDesign};

/** A faixa atual, normalizada (o backend fala snake_case, o evento camelCase). */
export type NowTrack={
 id:string;name:string;artist:string;album:string;art:string;url:string;
 durationMs:number;progressMs:number;timestamp:number;playing:boolean;
};

/** Porta própria da autorização — mesma do backend (spotify.rs REDIRECT). */
export const SPOTIFY_REDIRECT='http://127.0.0.1:43828/callback';
/** Onde o streamer cria o aplicativo gratuito que dá o Client ID. */
export const SPOTIFY_DASHBOARD='https://developer.spotify.com/dashboard';

/** Fontes aceitas — mesma lista do chat no overlay. */
export const SPOTIFY_FONTS=[
 'Segoe UI','Arial','Georgia','Verdana','Trebuchet MS','Courier New',
 'Inter','Montserrat','Poppins','Rubik','Orbitron','Bebas Neue',
 'Press Start 2P','Playfair Display','Rajdhani','Quicksand',
] as const;

/** Padrão espelhado em spotify.rs (design_defaults). */
export const DEFAULT_SPOTIFY_DESIGN:SpotifyDesign={
 layout:'card',cover:'square',coverGlow:false,coverBlur:false,
 showArt:true,showAlbum:true,showProgress:true,showTimes:true,
 bg:'#0f1319',titleColor:'#ffffff',artistColor:'#b9c0cc',albumColor:'#8a92a3',
 barBg:'#2a3140',barColor:'#1ed760',barHeight:6,barRadius:3,
 font:'Segoe UI',titleSize:16,artistSize:13,artSize:88,artRadius:10,
 width:460,padding:14,radius:14,
 empty:'hide',emptyText:'Nada tocando',emptyArtist:'',
 hideOnPause:true,pauseDelay:0,
 position:'bottom-left',
 transition:'fade',exitTransition:'fade',transitionMs:400,
};
export const DEFAULT_SPOTIFY:SpotifyConfig={enabled:false,clientId:'',design:DEFAULT_SPOTIFY_DESIGN};

const bool=(v:unknown,f:boolean)=>typeof v==='boolean'?v:f;
const str=(v:unknown,max:number,f:string)=>typeof v==='string'?v.slice(0,max):f;
const num=(v:unknown,min:number,max:number,f:number)=>typeof v==='number'&&Number.isFinite(v)?Math.min(max,Math.max(min,v)):f;
const color=(v:unknown,f:string):string=>(typeof v==='string'&&/^#[0-9a-fA-F]{3,8}$/.test(v)?v:f);
const pick=<T extends string>(v:unknown,list:readonly T[],fallback:T):T=>(typeof v==='string'&&list.includes(v as T)?v as T:fallback);

/** Rótulos das peles, capas e posições — usados pelo Estúdio e pelos testes. */
export const SPOTIFY_LAYOUTS:[SpotifyLayout,string][]=[['card','Card com capa'],['stacked','Empilhado'],['compact','Compacto'],['minimal','Só texto']];
export const SPOTIFY_COVERS:[SpotifyCover,string][]=[['square','Quadrada'],['circle','Redonda']];
export const SPOTIFY_POSITIONS:[SpotifyPosition,string][]=[
 ['bottom-left','Baixo à esquerda'],['bottom-right','Baixo à direita'],
 ['top-left','Topo à esquerda'],['top-right','Topo à direita'],['center','Centro'],
];
export const SPOTIFY_TRANSITIONS:[SpotifyTransition,string][]=[['fade','Pedar'],['slide','Subir'],['none','Sem animação']];

/** Aceita o JSON salvo (ou nada) e devolve uma configuração sempre utilizável. */
export function normalizeSpotify(v:unknown):SpotifyConfig{
 const o=(v&&typeof v==='object'?v:{}) as Record<string,unknown>;
 const d=(o.design&&typeof o.design==='object'?o.design:{}) as Record<string,unknown>;
 const dd=DEFAULT_SPOTIFY_DESIGN;
 return{
  enabled:bool(o.enabled,DEFAULT_SPOTIFY.enabled),
  clientId:str(o.clientId,128,DEFAULT_SPOTIFY.clientId),
  design:{
   layout:pick(d.layout,['card','compact','stacked','minimal'] as const,dd.layout),
   cover:pick(d.cover,['square','circle'] as const,dd.cover),
   coverGlow:bool(d.coverGlow,dd.coverGlow),coverBlur:bool(d.coverBlur,dd.coverBlur),
   showArt:bool(d.showArt,dd.showArt),showAlbum:bool(d.showAlbum,dd.showAlbum),
   showProgress:bool(d.showProgress,dd.showProgress),showTimes:bool(d.showTimes,dd.showTimes),
   bg:color(d.bg,dd.bg),titleColor:color(d.titleColor,dd.titleColor),
   artistColor:color(d.artistColor,dd.artistColor),albumColor:color(d.albumColor,dd.albumColor),
   barBg:color(d.barBg,dd.barBg),barColor:color(d.barColor,dd.barColor),
   barHeight:num(d.barHeight,2,24,dd.barHeight),barRadius:num(d.barRadius,0,12,dd.barRadius),
   font:pick(d.font,SPOTIFY_FONTS,dd.font),
   titleSize:num(d.titleSize,10,48,dd.titleSize),artistSize:num(d.artistSize,9,32,dd.artistSize),
   artSize:num(d.artSize,48,220,dd.artSize),artRadius:num(d.artRadius,0,60,dd.artRadius),
   width:num(d.width,240,960,dd.width),padding:num(d.padding,0,48,dd.padding),radius:num(d.radius,0,40,dd.radius),
   empty:pick(d.empty,['hide','message'] as const,dd.empty),emptyText:str(d.emptyText,64,dd.emptyText),
   emptyArtist:str(d.emptyArtist,64,dd.emptyArtist),
   hideOnPause:bool(d.hideOnPause,dd.hideOnPause),pauseDelay:num(d.pauseDelay,0,30,dd.pauseDelay),
   position:pick(d.position,['bottom-left','bottom-right','top-left','top-right','center'] as const,dd.position),
   transition:pick(d.transition,['fade','slide','none'] as const,dd.transition),
   exitTransition:pick(d.exitTransition,['fade','slide','none'] as const,dd.exitTransition),
   transitionMs:num(d.transitionMs,0,2000,dd.transitionMs),
  },
 };
}

/** Relógio m:ss — o mesmo formato que o backend escreve nas variáveis. */
export function fmtTime(ms:number):string{
 const total=Math.max(0,Math.floor(ms/1000));
 return`${Math.floor(total/60)}:${String(total%60).padStart(2,'0')}`;
}

/**
 * Normaliza a faixa atual vinda do `spotify.get` (snake_case) ou do evento
 * `spotify.now` (camelCase). Sem faixa ou sem nome: nada tocando (null).
 */
export function normalizeNow(v:unknown):NowTrack|null{
 if(!v||typeof v!=='object')return null;
 const o=v as Record<string,unknown>;
 const name=typeof o.name==='string'?o.name:'';
 if(!name)return null;
 const n=(k:string,camel:string,f=0)=>typeof o[k]==='number'?o[k] as number:typeof o[camel]==='number'?o[camel] as number:f;
 return{
  id:typeof o.id==='string'?o.id:'',
  name,
  artist:typeof o.artist==='string'?o.artist:'',
  album:typeof o.album==='string'?o.album:'',
  art:typeof o.art==='string'?o.art:'',
  url:typeof o.url==='string'?o.url:'',
  durationMs:n('duration_ms','durationMs'),
  progressMs:n('progress_ms','progressMs'),
  timestamp:n('timestamp','timestamp',Date.now()),
  playing:bool(o.playing,true),
 };
}

/** Posição decorrida, contando o tempo desde a última leitura (0,1 s no OBS). */
export function elapsed(t:NowTrack,at=Date.now()):number{
 if(!t.playing)return t.progressMs;
 return Math.min(t.durationMs,t.progressMs+Math.max(0,at-t.timestamp));
}

/** Hex com alfa para o brilho da capa — sem color-mix, compatível com o CEF antigo. */
export function hexA(hex:string,a:number):string{
 const m=/^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.exec(hex);
 if(!m)return`rgba(255,255,255,${a})`;
 let h=m[1];
 if(h.length===3)h=h.split('').map(c=>c+c).join('');
 return`rgba(${parseInt(h.slice(0,2),16)},${parseInt(h.slice(2,4),16)},${parseInt(h.slice(4,6),16)},${a})`;
}

/** As variáveis CSS que o cartão lê — a ponte entre o design e o desenho. */
export function spotifyVars(d:SpotifyDesign):Record<string,string>{
 return{
  '--sp-bg':d.bg,'--sp-title':d.titleColor,'--sp-artist':d.artistColor,'--sp-album':d.albumColor,
  '--sp-barbg':d.barBg,'--sp-bar':d.barColor,'--sp-glow':hexA(d.barColor,.5),
  '--sp-font':`"${d.font}",system-ui,sans-serif`,
  '--sp-tsize':d.titleSize+'px','--sp-asize':d.artistSize+'px',
  '--sp-art':d.artSize+'px','--sp-artr':(d.cover==='circle'?999:d.artRadius)+'px',
  '--sp-w':d.width+'px','--sp-pad':d.padding+'px','--sp-r':d.radius+'px',
  '--sp-bh':d.barHeight+'px','--sp-br':d.barRadius+'px','--sp-ms':d.transitionMs+'ms',
 };
}

/**
 * O cartão aparece? A mesma regra na prévia e no OBS: tocando sempre; na
 * pausa, some depois da espera (0 = na hora, como na Fase A); sem faixa,
 * segue o comportamento escolhido para o parado.
 */
export function spotifyShouldShow(d:SpotifyDesign,t:NowTrack|null,pausedAgeMs:number):boolean{
 if(t&&t.playing)return true;
 if(t)return !d.hideOnPause||pausedAgeMs<(d.pauseDelay||0)*1000;
 return d.empty==='message';
}

const NOTE_SVG='<svg viewBox="0 0 24 24" width="34%" height="34%" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>';

/** O CSS do cartão, injetado uma vez (mesmo costume de alerts.ts e chatOverlay.ts). */
export function ensureSpotifyStyle(){
 if(document.getElementById('bl-spy-style'))return;
 const s=document.createElement('style');s.id='bl-spy-style';
 s.textContent=`
 .bl-spy-host{position:absolute;inset:0;pointer-events:none;overflow:hidden}
 .bl-spy-host--screen{position:fixed}
 .bl-spy-slot{position:absolute}
 .bl-spy-slot[data-position=bottom-left]{left:18px;bottom:18px}
 .bl-spy-slot[data-position=bottom-right]{right:18px;bottom:18px}
 .bl-spy-slot[data-position=top-left]{left:18px;top:18px}
 .bl-spy-slot[data-position=top-right]{right:18px;top:18px}
 .bl-spy-slot[data-position=center]{inset:0;display:flex;align-items:center;justify-content:center}
 .bl-spy{position:relative;box-sizing:border-box;width:var(--sp-w);max-width:96vw;padding:var(--sp-pad);
  border-radius:var(--sp-r);background:var(--sp-bg);font-family:var(--sp-font);user-select:none;
  display:flex;gap:12px;align-items:center;overflow:hidden;transition:opacity .2s}
 .bl-spy[data-show=off]{display:none}
 .bl-spy[data-paused=on]{opacity:.72}
 .bl-spy-blur{position:absolute;inset:-42px;background:center/cover no-repeat;filter:blur(34px) saturate(1.4);opacity:.38;pointer-events:none}
 .bl-spy[data-blur=off] .bl-spy-blur{display:none}
 .bl-spy-art{position:relative;z-index:1;flex:0 0 auto;width:var(--sp-art);height:var(--sp-art);
  border-radius:var(--sp-artr);background:#00000055 center/cover no-repeat;display:flex;align-items:center;justify-content:center;color:#ffffff9c}
 .bl-spy[data-cover=circle] .bl-spy-art{border-radius:50%}
 .bl-spy[data-glow=on] .bl-spy-art{box-shadow:0 0 22px var(--sp-glow)}
 .bl-spy[data-showart=off] .bl-spy-art,.bl-spy[data-layout=minimal] .bl-spy-art{display:none}
 .bl-spy-meta{position:relative;z-index:1;min-width:0;flex:1;display:flex;flex-direction:column;gap:4px}
 .bl-spy-title{font-size:var(--sp-tsize);font-weight:600;color:var(--sp-title);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
 .bl-spy-artist{font-size:var(--sp-asize);color:var(--sp-artist);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
 .bl-spy-album{font-size:var(--sp-asize);color:var(--sp-album);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
 .bl-spy[data-album=off] .bl-spy-album,.bl-spy[data-state=empty] .bl-spy-album{display:none}
 .bl-spy-bar{margin-top:2px;height:var(--sp-bh);border-radius:var(--sp-br);background:var(--sp-barbg);overflow:hidden}
 .bl-spy[data-progress=off] .bl-spy-bar,.bl-spy[data-state=empty] .bl-spy-bar{display:none}
 .bl-spy-bar i{display:block;height:100%;width:0;border-radius:var(--sp-br);background:var(--sp-bar);transition:width .5s linear}
 .bl-spy-times{display:flex;justify-content:space-between;font-size:calc(var(--sp-asize) - 2px);color:var(--sp-album)}
 .bl-spy-times b,.bl-spy-times s{font-weight:400;text-decoration:none}
 .bl-spy[data-times=off] .bl-spy-times,.bl-spy[data-state=empty] .bl-spy-times{display:none}
 .bl-spy[data-layout=stacked]{flex-direction:column;align-items:stretch;text-align:center}
 .bl-spy[data-layout=stacked] .bl-spy-art{align-self:center}
 .bl-spy[data-layout=compact]{gap:8px}
 .bl-spy[data-layout=compact] .bl-spy-art{width:min(var(--sp-art),52px);height:min(var(--sp-art),52px)}
 .bl-spy[data-layout=minimal]{gap:6px}
 @keyframes bl-spy-in-fade{from{opacity:0}to{opacity:1}}
 @keyframes bl-spy-in-slide{from{opacity:0;transform:translateY(18px)}to{opacity:1;transform:none}}
 @keyframes bl-spy-out-fade{from{opacity:1}to{opacity:0}}
 @keyframes bl-spy-out-slide{from{opacity:1;transform:none}to{opacity:0;transform:translateY(18px)}}
 .bl-spy.bl-spy--in-fade{animation:bl-spy-in-fade var(--sp-ms) both}
 .bl-spy.bl-spy--in-slide{animation:bl-spy-in-slide var(--sp-ms) both}
 .bl-spy.bl-spy--out-fade{animation:bl-spy-out-fade var(--sp-ms) both}
 .bl-spy.bl-spy--out-slide{animation:bl-spy-out-slide var(--sp-ms) both}
 @media (prefers-reduced-motion:reduce){.bl-spy{animation:none!important}}
 `;
 document.head.appendChild(s);
}

/** O mesmo desenho na prévia do Estúdio e na fonte do OBS. */
export type SpotifyPlayer={
 /** Atualiza design e faixa reais. */
 update(design:SpotifyDesign,track:NowTrack|null):void;
 /** Mostra uma amostra por alguns segundos e volta ao estado real (selo ENSAIO fica por fora). */
 sample(track:NowTrack,ms?:number):void;
 /** Esconde por decisão (módulo desligado), ignorando o estado do parado. */
 clear():void;
 destroy():void;
};

export function spotifyPlayer(host:HTMLElement):SpotifyPlayer{
 ensureSpotifyStyle();
 const slot=document.createElement('div');slot.className='bl-spy-slot';
 const card=document.createElement('div');card.className='bl-spy';card.dataset.show='off';
 card.innerHTML=`<div class="bl-spy-blur"></div><div class="bl-spy-art" aria-hidden="true">${NOTE_SVG}</div><div class="bl-spy-meta"><div class="bl-spy-title"></div><div class="bl-spy-artist"></div><div class="bl-spy-album"></div><div class="bl-spy-bar"><i></i></div><div class="bl-spy-times"><b></b><s></s></div></div>`;
 slot.appendChild(card);host.appendChild(slot);
 const q=(sel:string)=>card.querySelector(sel) as HTMLElement;
 const el={blur:q('.bl-spy-blur'),art:q('.bl-spy-art'),title:q('.bl-spy-title'),artist:q('.bl-spy-artist'),album:q('.bl-spy-album'),fill:q('.bl-spy-bar i'),pass:q('.bl-spy-times b'),dur:q('.bl-spy-times s')};
 let design=DEFAULT_SPOTIFY_DESIGN;
 let track:NowTrack|null=null;
 let override:NowTrack|null=null,overrideUntil=0;
 let pausedAt=0,forced=false,shown=false;
 let leave=0,anim=0;
 const view=()=>override??track;

 function paint(){
  const t=view();
  const show=!forced&&spotifyShouldShow(design,t,pausedAt?Date.now()-pausedAt:0);
  card.dataset.layout=design.layout;card.dataset.cover=design.cover;
  card.dataset.showart=design.showArt?'on':'off';card.dataset.glow=design.coverGlow?'on':'off';
  card.dataset.blur=(design.coverBlur&&!!t?.art)?'on':'off';
  card.dataset.album=design.showAlbum?'on':'off';card.dataset.progress=design.showProgress?'on':'off';
  card.dataset.times=design.showTimes?'on':'off';
  card.dataset.state=t?'music':'empty';card.dataset.paused=(t&&!t.playing)?'on':'off';
  slot.dataset.position=design.position;
  const vars=spotifyVars(design);
  for(const k in vars)card.style.setProperty(k,vars[k]);
  el.title.textContent=t?t.name:design.emptyText;
  el.artist.textContent=t?t.artist:design.emptyArtist;
  el.album.textContent=t?t.album:'';
  el.art.style.backgroundImage=t?.art?`url("${t.art.replace(/"/g,'%22')}")`:'';
  if(t){const at=elapsed(t);el.fill.style.width=(t.durationMs>0?Math.min(100,at/t.durationMs*100):0)+'%';el.pass.textContent=fmtTime(at);el.dur.textContent=fmtTime(t.durationMs);}
  else{el.fill.style.width='0';el.pass.textContent='';el.dur.textContent='';}
  if(show&&!shown){shown=true;enter();}
  else if(!show&&shown){shown=false;leaveNow();}
 }
 function enter(){
  clearTimeout(leave);card.classList.remove('bl-spy--out-fade','bl-spy--out-slide');
  card.dataset.show='on';
  if(design.transition==='none'||design.transitionMs<=0)return;
  const cls='bl-spy--in-'+design.transition;card.classList.add(cls);
  clearTimeout(anim);anim=setTimeout(()=>card.classList.remove(cls),design.transitionMs+80);
 }
 function leaveNow(){
  clearTimeout(anim);card.classList.remove('bl-spy--in-fade','bl-spy--in-slide');
  const done=()=>{card.dataset.show='off';card.classList.remove('bl-spy--out-fade','bl-spy--out-slide')};
  if(design.exitTransition==='none'||design.transitionMs<=0){done();return}
  card.classList.add('bl-spy--out-'+design.exitTransition);
  clearTimeout(leave);leave=setTimeout(done,design.transitionMs+80);
 }
 const timer=window.setInterval(()=>{
  const t=view();
  if(t&&t.playing&&shown){const at=elapsed(t);el.fill.style.width=(t.durationMs>0?Math.min(100,at/t.durationMs*100):0)+'%';el.pass.textContent=fmtTime(at);}
  // Amostra vencida: volta ao estado real com um pintar novo (textos inclusive).
  if(override&&Date.now()>=overrideUntil){override=null;paint();return;}
  const want=!forced&&spotifyShouldShow(design,view(),pausedAt?Date.now()-pausedAt:0);
  if(want!==shown){shown=want;want?enter():leaveNow();}
 },500);
 return{
  update(d,t){
   forced=false;
   if(t&&!t.playing&&pausedAt===0)pausedAt=Date.now();
   if(!t||t.playing)pausedAt=0;
   design=d;track=t;paint();
  },
  sample(t,ms=6000){override=t;overrideUntil=Date.now()+ms;paint();},
  clear(){forced=true;paint();},
  destroy(){clearInterval(timer);clearTimeout(leave);clearTimeout(anim);slot.remove();},
 };
}
