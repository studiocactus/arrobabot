/**
 * Configuração e leitura da música tocando — compartilhadas entre a tela
 * Spotify, a prévia do Estúdio (Fase B: overlay) e o catálogo de variáveis.
 * O backend transmite a faixa reduzida; aqui só se normaliza o que veio do
 * banco ou do evento, sempre com padrão para o que faltar.
 */

export type SpotifyLayout='card'|'compact'|'stacked';
export type SpotifyEmpty='hide'|'message';
export type SpotifyTransition='fade'|'slide'|'none';

export type SpotifyDesign={
 layout:SpotifyLayout;
 showArt:boolean;showAlbum:boolean;showProgress:boolean;showTimes:boolean;
 bg:string;titleColor:string;artistColor:string;albumColor:string;
 barBg:string;barColor:string;barHeight:number;barRadius:number;
 font:string;titleSize:number;artistSize:number;artSize:number;artRadius:number;
 width:number;padding:number;radius:number;
 empty:SpotifyEmpty;emptyText:string;
 transition:SpotifyTransition;transitionMs:number;
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
 layout:'card',
 showArt:true,showAlbum:true,showProgress:true,showTimes:true,
 bg:'#0f1319',titleColor:'#ffffff',artistColor:'#b9c0cc',albumColor:'#8a92a3',
 barBg:'#2a3140',barColor:'#1ed760',barHeight:6,barRadius:3,
 font:'Segoe UI',titleSize:16,artistSize:13,artSize:88,artRadius:10,
 width:460,padding:14,radius:14,
 empty:'hide',emptyText:'Nada tocando',
 transition:'fade',transitionMs:400,
};
export const DEFAULT_SPOTIFY:SpotifyConfig={enabled:false,clientId:'',design:DEFAULT_SPOTIFY_DESIGN};

const bool=(v:unknown,f:boolean)=>typeof v==='boolean'?v:f;
const str=(v:unknown,max:number,f:string)=>typeof v==='string'?v.slice(0,max):f;
const num=(v:unknown,min:number,max:number,f:number)=>typeof v==='number'&&Number.isFinite(v)?Math.min(max,Math.max(min,v)):f;
const color=(v:unknown,f:string):string=>(typeof v==='string'&&/^#[0-9a-fA-F]{3,8}$/.test(v)?v:f);
const pick=<T extends string>(v:unknown,list:readonly T[],fallback:T):T=>(typeof v==='string'&&list.includes(v as T)?v as T:fallback);

/** Aceita o JSON salvo (ou nada) e devolve uma configuração sempre utilizável. */
export function normalizeSpotify(v:unknown):SpotifyConfig{
 const o=(v&&typeof v==='object'?v:{}) as Record<string,unknown>;
 const d=(o.design&&typeof o.design==='object'?o.design:{}) as Record<string,unknown>;
 const dd=DEFAULT_SPOTIFY_DESIGN;
 return{
  enabled:bool(o.enabled,DEFAULT_SPOTIFY.enabled),
  clientId:str(o.clientId,128,DEFAULT_SPOTIFY.clientId),
  design:{
   layout:pick(d.layout,['card','compact','stacked'] as const,dd.layout),
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
   transition:pick(d.transition,['fade','slide','none'] as const,dd.transition),
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
