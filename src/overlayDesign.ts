/**
 * Estúdio do overlay de meta — fonte única de verdade.
 *
 * Este módulo é consumido por três lugares que precisam bater pixel a pixel:
 *  - `OverlayPreview.tsx` (prévia dentro do BotLive),
 *  - `overlayApp.ts` (o overlay servido em http://127.0.0.1:PORT/overlay e no arquivo examples/),
 *  - o teste de renderer em tests/ui/overlay.spec.ts.
 * O CSS é escopado em `.bl-ov` para nunca vazar para o restante do aplicativo.
 */

export type Layout='bar'|'row'|'panel'|'ring';
export type Align='left'|'center'|'right';
export type VAlign='top'|'center'|'bottom';

export type OverlayDesign={
 template:string;
 layout:Layout;align:Align;valign:VAlign;
 width:number;
 fillStart:string;fillEnd:string;track:string;text:string;muted:string;
 panel:string;border:string;borderWidth:number;
 radius:number;barHeight:number;
 font:string;fontSize:number;fontWeight:number;uppercase:boolean;tracking:number;
 glow:number;shadow:number;
 animate:boolean;pulse:boolean;celebrate:boolean;
};

export type GoalModel={title:string;current:number;target:number;hidden?:boolean};

/** Nó da árvore do overlay. `k` identifica textos atualizados sem reconstruir o DOM. */
export type Spec={tag?:string;c?:string;k?:string;t?:string;a?:Record<string,string>;ch?:Spec[]};

const SYSTEM_FONTS=['Segoe UI','Arial','Georgia','Verdana','Trebuchet MS','Courier New'];
const GOOGLE_FONTS=['Inter','Montserrat','Poppins','Rubik','Orbitron','Bebas Neue','Press Start 2P','Playfair Display','Rajdhani','Quicksand'];
export const fontGroups=[{label:'Fontes do sistema',fonts:SYSTEM_FONTS},{label:'Google Fonts (exigem internet no OBS)',fonts:GOOGLE_FONTS}];
const googleSet=new Set(GOOGLE_FONTS);

type Template={name:string;hint:string;design:OverlayDesign};

/** Modelos: cada um já traz cores, forma, fonte e efeitos prontos. */
export const TEMPLATES:Record<string,Template>={
 clean:{name:'Clean',hint:'Verde vivo, sem moldura',design:{
  template:'clean',layout:'bar',align:'center',valign:'bottom',width:80,
  fillStart:'#b7ef72',fillEnd:'#5bc46b',track:'#1b2634',text:'#e8edf4',muted:'#9fb0c3',
  panel:'#101721dd',border:'#2a3646',borderWidth:1,radius:10,barHeight:26,
  font:'Segoe UI',fontSize:18,fontWeight:700,uppercase:false,tracking:0,
  glow:25,shadow:45,animate:false,pulse:true,celebrate:true}},
 aurora:{name:'Aurora',hint:'Vidro fosco com gradiente frio',design:{
  template:'aurora',layout:'panel',align:'center',valign:'center',width:62,
  fillStart:'#22d3ee',fillEnd:'#8b5cf6',track:'#1e293b99',text:'#eef4ff',muted:'#a9b7d4',
  panel:'#0b1120b3',border:'#7dd3fc55',borderWidth:1,radius:20,barHeight:30,
  font:'Inter',fontSize:18,fontWeight:700,uppercase:false,tracking:0.3,
  glow:55,shadow:55,animate:true,pulse:true,celebrate:true}},
 neon:{name:'Neon',hint:'Magenta e ciano brilhando',design:{
  template:'neon',layout:'bar',align:'left',valign:'bottom',width:58,
  fillStart:'#ff2fd6',fillEnd:'#00e5ff',track:'#12081d',text:'#f8f4ff',muted:'#c3aede',
  panel:'#0a0a12e6',border:'#ff2fd666',borderWidth:2,radius:6,barHeight:24,
  font:'Orbitron',fontSize:17,fontWeight:800,uppercase:true,tracking:2,
  glow:85,shadow:40,animate:true,pulse:true,celebrate:true}},
 sunset:{name:'Sunset',hint:'Pílula quente de laranja a rosa',design:{
  template:'sunset',layout:'row',align:'center',valign:'top',width:72,
  fillStart:'#ff9a3c',fillEnd:'#ff2e93',track:'#3b1420',text:'#fff4ea',muted:'#ffcfb3',
  panel:'#2a0f1ce6',border:'#ff9a3c55',borderWidth:1,radius:48,barHeight:22,
  font:'Poppins',fontSize:17,fontWeight:600,uppercase:false,tracking:0.5,
  glow:45,shadow:60,animate:true,pulse:true,celebrate:true}},
 retro:{name:'Retro',hint:'Bloco amarelo, cantos retos',design:{
  template:'retro',layout:'bar',align:'right',valign:'top',width:46,
  fillStart:'#ffe14d',fillEnd:'#f47b20',track:'#241f10',text:'#fff6d6',muted:'#d9c887',
  panel:'#12100af2',border:'#ffe14d',borderWidth:3,radius:0,barHeight:28,
  font:'Press Start 2P',fontSize:14,fontWeight:400,uppercase:false,tracking:0,
  glow:20,shadow:70,animate:false,pulse:true,celebrate:true}},
 royal:{name:'Royal',hint:'Painel profundo com ouro',design:{
  template:'royal',layout:'panel',align:'center',valign:'bottom',width:66,
  fillStart:'#f5c542',fillEnd:'#d99a12',track:'#231a0e',text:'#fff7e2',muted:'#d8c391',
  panel:'#14101ff2',border:'#f5c54288',borderWidth:1,radius:14,barHeight:28,
  font:'Bebas Neue',fontSize:22,fontWeight:700,uppercase:true,tracking:3,
  glow:50,shadow:65,animate:true,pulse:true,celebrate:true}},
};
export const TEMPLATE_IDS=Object.keys(TEMPLATES);

/** Design usado quando o perfil ainda não salvou nada (= modelo Clean). */
export function defaultDesign():OverlayDesign{return {...TEMPLATES.clean.design}}

const num=(v:unknown,def:number,min:number,max:number)=>{const n=typeof v==='number'&&Number.isFinite(v)?v:def;return Math.min(max,Math.max(min,n))};
const WEIGHTS_OK=[300,400,500,600,700,800,900];
const snapWeight=(v:unknown,def:number)=>{const w=Math.round(num(v,def,100,900));return WEIGHTS_OK.includes(w)?w:700};
const str=(v:unknown,def:string)=>typeof v==='string'&&v?v:def;
const bool=(v:unknown,def:boolean)=>typeof v==='boolean'?v:def;
const oneOf=<T extends string>(v:unknown,def:T,allowed:readonly T[]):T=>typeof v==='string'&&(allowed as readonly string[]).includes(v)?v as T:def;
const HEX=/^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;
const color=(v:unknown,def:string)=>typeof v==='string'&&HEX.test(v)?v:def;
/** Mesmo permitido do backend (livepix.rs): letras, dígitos e ' -,.&+_ ; máximo 40. */
const FAMILY=/^[\p{L}\p{N} '\-,.&+_]{1,40}$/u;
const family=(v:unknown,def:string)=>typeof v==='string'&&FAMILY.test(v)?v:def;

/** Normaliza um design vindo do banco (ou de uma versão antiga) sem estourar faixas. */
export function normalizeDesign(v:unknown):OverlayDesign{
 const d=(v&&typeof v==='object'?v:{}) as Partial<Record<keyof OverlayDesign,unknown>>;
 const base=defaultDesign();
 return {
  template:str(d.template,base.template).slice(0,24),
  layout:oneOf(d.layout,base.layout,['bar','row','panel','ring'] as const),
  align:oneOf(d.align,base.align,['left','center','right'] as const),
  valign:oneOf(d.valign,base.valign,['top','center','bottom'] as const),
  width:num(d.width,base.width,20,100),
  fillStart:color(d.fillStart,base.fillStart),fillEnd:color(d.fillEnd,base.fillEnd),
  track:color(d.track,base.track),text:color(d.text,base.text),muted:color(d.muted,base.muted),
  panel:color(d.panel,base.panel),border:color(d.border,base.border),
  borderWidth:num(d.borderWidth,base.borderWidth,0,6),
  radius:num(d.radius,base.radius,0,48),
  barHeight:num(d.barHeight,base.barHeight,4,64),
  font:family(d.font,base.font),
  fontSize:num(d.fontSize,base.fontSize,10,48),
  fontWeight:snapWeight(d.fontWeight,base.fontWeight),
  uppercase:bool(d.uppercase,base.uppercase),
  tracking:num(d.tracking,base.tracking,-2,10),
  glow:num(d.glow,base.glow,0,100),
  shadow:num(d.shadow,base.shadow,0,100),
  animate:bool(d.animate,base.animate),
  pulse:bool(d.pulse,base.pulse),
  celebrate:bool(d.celebrate,base.celebrate),
 };
}

/** #rgb/#rrggbb/#rrggbbaa -> rgba() com o alfa pedido; sombras e brilhos calculados no TS. */
function alpha(hex:string,a:number){
 let s=hex.replace('#','');
 if(s.length===3||s.length===4)s=[...s].map(c=>c+c).join('');
 const r=parseInt(s.slice(0,2),16)||0,g=parseInt(s.slice(2,4),16)||0,b=parseInt(s.slice(4,6),16)||0;
 return `rgba(${r},${g},${b},${Math.max(0,Math.min(1,a)).toFixed(3)})`;
}

/** Variáveis CSS consumidas pelo CSS do overlay e pela prévia. */
export function designVars(d:OverlayDesign):Record<string,string>{
 const g=d.glow/100,s=d.shadow/100;
 const shadows:string[]=[];
 if(s>0.01)shadows.push(`0 ${Math.round(6+14*s)}px ${Math.round(20+40*s)}px rgba(0,0,0,${(0.18+0.4*s).toFixed(2)})`);
 if(g>0.01)shadows.push(`0 0 ${Math.round(14+26*g)}px ${alpha(d.fillEnd,0.45*g)}`);
 const fillShadow=g>0.01?`0 0 ${Math.round(6+18*g)}px ${alpha(d.fillEnd,0.6*g)}`:'none';
 return {
  '--bl-fill-a':d.fillStart,'--bl-fill-b':d.fillEnd,'--bl-track-color':d.track,
  '--bl-text':d.text,'--bl-muted':d.muted,'--bl-panel':d.panel,'--bl-border':d.border,
  '--bl-bw':`${Math.round(d.borderWidth*2)/2}px`,'--bl-radius':`${Math.round(d.radius)}px`,
  '--bl-bar-h':`${Math.round(d.barHeight)}px`,
  '--bl-font':`"${d.font}", "Segoe UI", system-ui, sans-serif`,
  '--bl-fs':`${Math.round(d.fontSize)}px`,'--bl-fw':`${Math.round(d.fontWeight)}`,
  '--bl-transform':d.uppercase?'uppercase':'none','--bl-ls':`${Math.round(d.tracking*10)/10}px`,
  '--bl-card-shadow':shadows.join(', ')||'none','--bl-fill-shadow':fillShadow,
  '--bl-w':`${Math.round(d.width)}%`,
  '--bl-margin':d.align==='center'?'auto':(d.align==='left'?'0 auto':'auto 0'),
  '--bl-vjustify':d.valign==='top'?'flex-start':(d.valign==='bottom'?'flex-end':'center'),
 };
}

export function goalPercent(m:GoalModel):number{
 if(!m.target||m.target<=0)return 0;
 return Math.max(0,Math.min(100,Math.round((m.current*100)/m.target)));
}

const brl=(cents:number)=>'R$ '+(Math.max(0,cents)/100).toLocaleString('pt-BR',{minimumFractionDigits:2,maximumFractionDigits:2});

/** Textos por chave: usados pela prévia (React) e pelo overlay (DOM direto). */
export function keyTexts(m:GoalModel):Record<string,string>{
 return {
  title:m.title.trim()||'Meta',
  pct:goalPercent(m)+'%',
  amounts:`${brl(m.current)} / ${brl(m.target)}`,
  current:brl(m.current),
  target:`de ${brl(m.target)}`,
 };
}

const head=(...ch:Spec[]):Spec=>({c:'bl-ov-head',ch});
const track=():Spec=>({c:'bl-ov-track',ch:[{c:'bl-ov-fill'}]});
const foot=(...ch:Spec[]):Spec=>({c:'bl-ov-foot',ch});
const T=(k:string,cls:string):Spec=>({c:cls,k,t:''});

/** Árvore completa do overlay para o design + textos informados. */
export function overlayTree(d:OverlayDesign,texts:Record<string,string>):Spec{
 const withText=(n:Spec):Spec=>n.k?{...n,t:texts[n.k]??''}:(n.ch?{...n,ch:n.ch.map(withText)}:n);
 let card:Spec;
 if(d.layout==='row'){
  card={c:'bl-ov-card',ch:[head({c:'bl-ov-title',k:'title',t:''},{c:'bl-ov-amounts',k:'amounts',t:''},{c:'bl-ov-pct',k:'pct',t:''}),track()]};
 }else if(d.layout==='panel'){
  card={c:'bl-ov-card',ch:[{c:'bl-ov-hero',k:'current',t:''},{c:'bl-ov-sub',k:'target',t:''},track(),foot({c:'bl-ov-title',k:'title',t:''},{c:'bl-ov-pct',k:'pct',t:''})]};
 }else if(d.layout==='ring'){
  card={c:'bl-ov-card',ch:[
   {c:'bl-ov-ringwrap',ch:[
    {tag:'svg',c:'bl-ov-ring-svg',a:{viewBox:'0 0 100 100','aria-hidden':'true'},ch:[
     {tag:'defs',ch:[{tag:'linearGradient',a:{id:'bl-ov-grad',x1:'0',y1:'0',x2:'1',y2:'1'},ch:[{tag:'stop',c:'bl-ov-stop-a'},{tag:'stop',c:'bl-ov-stop-b',a:{offset:'1'}}]}]},
     {tag:'circle',c:'bl-ov-ring-track',a:{cx:'50',cy:'50',r:'42'}},
     {tag:'circle',c:'bl-ov-ring-fill',a:{cx:'50',cy:'50',r:'42',pathLength:'100'}},
    ]},
    {c:'bl-ov-ring-center',ch:[{c:'bl-ov-ring-pct',k:'pct',t:''},{c:'bl-ov-title bl-ov-ring-title',k:'title',t:''}]},
   ]},
   foot({c:'bl-ov-amounts',k:'amounts',t:''}),
  ]};
 }else{
  card={c:'bl-ov-card',ch:[head({c:'bl-ov-title',k:'title',t:''},{c:'bl-ov-pct',k:'pct',t:''}),track(),foot({c:'bl-ov-amounts',k:'amounts',t:''})]};
 }
 return withText({c:`bl-ov bl-ov-layout-${d.layout}`+(d.animate?' is-anim':''),ch:[
  {c:'bl-ov-box',ch:[
   card,
   {c:'bl-ov-conf',ch:Array.from({length:10},(_,i)=>({tag:'i',c:`bl-ov-conf-${i}`}))},
  ]},
 ]});
}

let styleEl:HTMLStyleElement|null=null;
/** Injeta o CSS do overlay uma única vez no documento atual. */
export function ensureOverlayStyle(){
 if(styleEl&&styleEl.isConnected)return;
 styleEl=document.createElement('style');
 styleEl.id='bl-ov-style';
 styleEl.textContent=OVERLAY_CSS;
 document.head.appendChild(styleEl);
}

const fontLinks=new Set<string>();
/** Carrega a folha do Google Fonts uma vez por família (fontes do sistema são locais). */
export function ensureFont(familyName:string){
 if(!googleSet.has(familyName)||fontLinks.has(familyName)||typeof document==='undefined')return;
 fontLinks.add(familyName);
 const link=document.createElement('link');
 link.rel='stylesheet';
 link.href='https://fonts.googleapis.com/css2?family='+familyName.replace(/ /g,'+')+'&display=swap';
 document.head.appendChild(link);
}

/** CSS escopado — só regras sob `.bl-ov`, sem tocar em body/html do aplicativo. */
export const OVERLAY_CSS=`
.bl-ov,.bl-ov *{box-sizing:border-box}
.bl-ov{position:absolute;inset:0;display:flex;flex-direction:column;justify-content:var(--bl-vjustify);
 padding:2.5%;font-family:var(--bl-font);color:var(--bl-text);font-size:var(--bl-fs);font-weight:var(--bl-fw);
 letter-spacing:var(--bl-ls);text-transform:var(--bl-transform);-webkit-font-smoothing:antialiased}
.bl-ov.is-off{display:none}
.bl-ov-box{position:relative;width:var(--bl-w);max-width:100%;margin-inline:var(--bl-margin)}
.bl-ov-card{position:relative;display:flex;flex-direction:column;gap:calc(var(--bl-fs)*.5);
 padding:calc(var(--bl-fs)*.8) calc(var(--bl-fs)*1.05);background:var(--bl-panel);
 border:var(--bl-bw) solid var(--bl-border);border-radius:var(--bl-radius);
 box-shadow:var(--bl-card-shadow);backdrop-filter:blur(14px);-webkit-backdrop-filter:blur(14px)}
.bl-ov-head{display:flex;align-items:baseline;justify-content:space-between;gap:14px}
.bl-ov-title{font-size:var(--bl-fs);font-weight:var(--bl-fw);margin:0;min-width:0;
 overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.bl-ov-pct{color:var(--bl-muted);font-variant-numeric:tabular-nums;white-space:nowrap;flex:0 0 auto}
.bl-ov-track{position:relative;height:var(--bl-bar-h);border-radius:var(--bl-radius);
 background:var(--bl-track-color);overflow:hidden}
.bl-ov-fill{height:100%;width:calc(var(--p,0)*1%);border-radius:inherit;
 background:linear-gradient(90deg,var(--bl-fill-a),var(--bl-fill-b));background-size:200% 100%;
 box-shadow:var(--bl-fill-shadow);transition:width .9s cubic-bezier(.22,.9,.25,1)}
.bl-ov.is-anim .bl-ov-fill{animation:bl-ov-slide 3.2s linear infinite}
.bl-ov-foot{display:flex;align-items:baseline;justify-content:space-between;gap:14px;
 color:var(--bl-muted);font-size:calc(var(--bl-fs)*.76)}
.bl-ov-amounts{font-variant-numeric:tabular-nums;white-space:nowrap;min-width:0;
 overflow:hidden;text-overflow:ellipsis}
.bl-ov-layout-row .bl-ov-card{gap:calc(var(--bl-fs)*.4)}
.bl-ov-layout-row .bl-ov-amounts{margin-left:.7em;color:var(--bl-muted);flex:1 1 auto}
.bl-ov-layout-row .bl-ov-pct{margin-left:auto}
.bl-ov-layout-row .bl-ov-track{height:calc(var(--bl-bar-h)*.55)}
.bl-ov-layout-panel .bl-ov-hero{font-size:calc(var(--bl-fs)*1.95);line-height:1.05;
 font-weight:calc(var(--bl-fw) + 100);letter-spacing:-.5px;font-variant-numeric:tabular-nums}
.bl-ov-layout-panel .bl-ov-sub{color:var(--bl-muted);font-size:calc(var(--bl-fs)*.8);
 margin-top:calc(var(--bl-fs)*-.25)}
.bl-ov-ringwrap{position:relative;display:flex;align-items:center;justify-content:center;
 padding:calc(var(--bl-fs)*.2) 0}
.bl-ov-ring-svg{width:calc(var(--bl-fs)*6.4);height:calc(var(--bl-fs)*6.4);transform:rotate(-90deg);
 overflow:visible}
.bl-ov-ring-track{fill:none;stroke:var(--bl-track-color);stroke-width:7}
.bl-ov-ring-fill{fill:none;stroke:url(#bl-ov-grad);stroke-width:7;stroke-linecap:round;
 stroke-dasharray:calc(var(--p,0)) calc(100 - var(--p,0));transition:stroke-dasharray .9s cubic-bezier(.22,.9,.25,1)}
.bl-ov-stop-a{stop-color:var(--bl-fill-a)}
.bl-ov-stop-b{stop-color:var(--bl-fill-b)}
.bl-ov-ring-center{position:absolute;inset:0;display:flex;flex-direction:column;align-items:center;
 justify-content:center;text-align:center;gap:1px}
.bl-ov-ring-pct{font-size:calc(var(--bl-fs)*1.35);line-height:1;font-variant-numeric:tabular-nums}
.bl-ov-layout-ring .bl-ov-title{font-size:calc(var(--bl-fs)*.72);color:var(--bl-muted);
 white-space:normal;max-width:calc(var(--bl-fs)*6.2)}
.bl-ov-layout-ring .bl-ov-foot{justify-content:center}
.bl-ov-conf{position:absolute;inset:0;pointer-events:none;overflow:hidden}
.bl-ov-conf i{position:absolute;top:-8%;width:9px;height:15px;border-radius:2px;opacity:0;
 display:block;background:var(--bl-fill-a)}
.bl-ov-conf-0{left:6%}.bl-ov-conf-1{left:16%;background:var(--bl-fill-b);animation-delay:.05s}
.bl-ov-conf-2{left:27%;animation-delay:.12s}.bl-ov-conf-3{left:38%;background:var(--bl-fill-b);animation-delay:.02s}
.bl-ov-conf-4{left:49%;animation-delay:.16s}.bl-ov-conf-5{left:60%;background:var(--bl-fill-b);animation-delay:.09s}
.bl-ov-conf-6{left:71%;animation-delay:.2s}.bl-ov-conf-7{left:80%;background:var(--bl-fill-b);animation-delay:.04s}
.bl-ov-conf-8{left:89%;animation-delay:.14s}.bl-ov-conf-9{left:95%;background:var(--bl-fill-b);animation-delay:.07s}
.bl-ov.is-celebrate .bl-ov-conf i{animation:bl-ov-conf .95s ease-out forwards}
.bl-ov.is-celebrate .bl-ov-card{animation:bl-ov-pop .7s cubic-bezier(.34,1.56,.64,1)}
.bl-ov.is-celebrate .bl-ov-fill{animation:bl-ov-flash .8s ease-out}
.bl-ov.is-gain .bl-ov-fill{animation:bl-ov-gain .75s ease-out}
.bl-ov.is-gain .bl-ov-card{animation:bl-ov-flash .75s ease-out}
@keyframes bl-ov-slide{0%{background-position:0% 50%}100%{background-position:200% 50%}}
@keyframes bl-ov-gain{0%{filter:brightness(1)}30%{filter:brightness(1.7)}100%{filter:brightness(1)}}
@keyframes bl-ov-flash{0%{box-shadow:var(--bl-card-shadow)}40%{box-shadow:var(--bl-card-shadow),0 0 34px var(--bl-fill-shadow)}100%{box-shadow:var(--bl-card-shadow)}}
@keyframes bl-ov-pop{0%{transform:scale(1)}35%{transform:scale(1.035)}100%{transform:scale(1)}}
@keyframes bl-ov-conf{0%{opacity:1;transform:translateY(0) rotate(0deg)}100%{opacity:0;transform:translateY(70px) rotate(240deg)}}
@media (prefers-reduced-motion: reduce){
 .bl-ov,.bl-ov *{animation:none!important;transition:none!important}
}
`;
