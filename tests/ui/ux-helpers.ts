import type {Page} from '@playwright/test';
import {tmpdir} from 'os';
import {join} from 'path';

/** Pasta de saída fora do repositório: nenhum artefato de teste é versionado. */
export const OUT=join(tmpdir(),'botlive-ux');

export const profile={id:'p1',name:'Canal de teste',platform:'twitch',channel:'cactus',channelId:'1',botId:'',clientId:'',blocklist:[],topics:[],editors:[],ai:{},modules:{}};

const act=(kind:string,text='',extra:Record<string,unknown>={})=>({kind,text,target:'',value:0,condition:'',aiAnchor:'',aiKnowledge:'',aiLength:'',aiStyle:'',aiNoRepeat:null,...extra});

export const flows=[
 {id:'f1',profileId:'p1',name:'Boas-vindas',enabled:true,audio:'',audioVolume:1,sendType:'chat',sendColor:'primary',counter:false,
  trigger:{kind:'command',pattern:'!bemvindo',permission:'everyone',cooldown:5,userCooldown:15},
  actions:[act('chat','Olá {{viewer.name}}'),act('delay',''),act('punish','',{punish:'timeout'})]},
 {id:'f2',profileId:'p1',name:'Alerta da live com um nome bem longo para forçar corte de texto',enabled:false,audio:'',audioVolume:1,sendType:'chat',sendColor:'primary',counter:false,
  trigger:{kind:'contains',pattern:'!alerta, !alerta2, !alerta3',permission:'moderator',cooldown:60,userCooldown:120},
  actions:[act('condition','',{condVar:'reward.cost',condOp:'greater_or_equal',condValue:'100',condFalse:'stop'}),act('chat','Oi {{viewer.name}}'),act('overlay','Aparece na OBS')]}
];

export const NAV=['Visão geral','Perfis de bot','Discord','Rótulos OBS','Comandos','Timers','Respostas e sons','Sons','OBS Studio','Voicemod','Contadores','Automações','Inteligência artificial','Memórias','Comunidade','Biblioteca de presets','Histórico','Estatísticas'];

const LOGS=[
 {id:'l1',profileId:'p1',kind:'run',message:'Execução concluída com sucesso no fluxo de boas-vindas',status:'success',at:'2026-10-07T10:00:00Z'},
 {id:'l2',profileId:'p1',kind:'obs',message:'Fonte mutada',status:'info',at:'2026-10-07T09:00:00Z'},
 {id:'l3',profileId:'p1',kind:'obs',message:'Não foi possível alcançar o OBS em ws://127.0.0.1:4455. Abra o OBS e ative o WebSocket do OBS em Ferramentas.',status:'error',at:'2026-10-07T08:00:00Z'}
];

/** Forma padrão que o backend devolve para `community` (struct Community). */
export const community={
 lastChat:0,lastTrivia:0,
 balances:{u1:1240,u2:980,u3:640},
 names:{u1:'ana_gamer',u2:'pedro_dev',u3:'lulu_oficial'},
 queue:['u1','u2'],
 songs:[{user:'u1',url:'https://www.youtube.com/watch?v=dQw4w9WgXcQ'}],
 raffle:{open:false,keyword:'',price:0,entries:{},weights:{},winner:''},
 prediction:{open:false,title:'',options:[],bets:{},result:null},
 shop:[
  {id:'i1',name:'Escolher o próximo desafio',cost:100,stock:10},
  {id:'i2',name:'Mensagem em destaque com um nome bem comprido para testar o corte',cost:50,stock:3},
  {id:'i3',name:'Jogar com o streamer',cost:200,stock:10}
 ],
 redemptions:[{user:'ana_gamer',item:'Mensagem em destaque'}],
 lastEarned:{},gameAt:{},trivia:null,duel:null,bingo:null
};

/** Forma padrão que o backend devolve para `stats`. */
export const stats={
 messages:1248,actions:312,followers:17,
 hours:[{hour:'18:00',count:42},{hour:'19:00',count:88},{hour:'20:00',count:120}],
 commands:[{name:'!pontos',count:64},{name:'!so',count:31},{name:'!bemvindo',count:22}]
};

/** Forma padrão que o backend devolve para `voicemod.get`. */
export const voicemod={
 status:{phase:'connected',message:'Conectado ao Voicemod na porta 39273.',port:39273,
  voices:[
   {id:'nofx',friendlyName:'Efeito desligado',enabled:true,favorited:false,isNew:false,isCustom:false},
   {id:'robot',friendlyName:'Robô',enabled:true,favorited:true,isNew:false,isCustom:false},
   {id:'alien',friendlyName:'Alien com um nome comprido para forçar o corte',enabled:true,favorited:false,isNew:true,isCustom:false},
   {id:'blocked',friendlyName:'Voz bloqueada pela licença',enabled:false,favorited:false,isNew:false,isCustom:false}
  ],
  currentVoice:'nofx',currentName:'Efeito desligado',voiceChanger:false,hearMyself:false,license:'free',attempts:0},
 test:{active:false,phase:'',voiceId:'',voiceName:'',seconds:0,remainingMs:0,interrupted:false,manual:false},
 outcome:{kind:'',detail:''},
 hasKey:true,keyLen:21,defaultSecs:10,minSecs:1,maxSecs:60
};

export async function mock(page:Page){
 await page.addInitScript(arg=>{
  const w=window as unknown as Record<string,unknown>;
  const st={ops:[] as string[],unknown:[] as string[]};
  w.__ux=st;
  const lists=['logs','flows','presets','notes','knowledge.list','command.counters','backup.list','variables.list','twitch.rewards','labels.list','notes.list','timers','commands','counters','sounds','phrases'];
  Object.assign(w,{isTauri:true,
   __TAURI_EVENT_PLUGIN_INTERNALS__:{registerListener:()=>{},unregisterListener:()=>{}},
   __TAURI_INTERNALS__:{transformCallback:()=>1,unregisterCallback:()=>{},
    invoke:async(cmd:string,payload:{op:string;args?:Record<string,unknown>})=>{
     if(cmd!=='rpc')return 1;
     const op=payload.op;st.ops.push(op);
     switch(op){
      case 'snapshot':return {profiles:[arg.profile],logs:arg.logs,statuses:{p1:'connected'},theme:'dark',accent:'',apiPort:0,dataDir:'C:/dados'};
      case 'flows':return arg.flows;
      case 'command.counters':return {c:{v:12}};
      case 'module.config.get':return {};
      case 'settings.get':return {theme:'dark',apiPort:0,apiToken:'',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};
      case 'backup.get':return {enabled:false,folder:'',keep:7};
      case 'backup.list':return [];
      case 'presets':return [];
      case 'logs':return arg.logs;
      case 'stats':return arg.stats;
      case 'community':return arg.community;
      case 'obs.get':return {config:{enabled:false,host:'127.0.0.1',port:4455},hasPassword:false};
      case 'obs.status':return null;
      case 'obs.discover':return null;
      case 'voicemod.get':return arg.voicemod;
      case 'voice.status':return {listening:false,error:'',port:0};
      case 'secret.status':return {ai:false,bot:false,channel:false};
      case 'discord.status':return {online:false};
      case 'access.status':return {user:'owner',enabled:false};
      case 'labels.list':return [];
      case 'chatExtras.get':return {phrases:[],sounds:[]};
      case 'update.check':return {available:false};
      default:st.unknown.push(op);return lists.includes(op)?[]:null;
     }
    }}} as unknown as void);
 },{profile:profile,flows:flows,logs:LOGS,community:community,stats:stats,voicemod:voicemod});
}

export type Issue={k:string;sel:string;txt:string;extra:string};

/** Detecta texto cortado, caixa estourando o pai e elementos fora da viewport. */
export function auditPage():Issue[] {
 const out:Issue[]=[];
 const path=(el:Element)=>{const p:string[]=[];let n:Element|null=el;for(let i=0;n&&i<5;i++,n=n.parentElement){p.unshift(n.tagName.toLowerCase()+(n.className&&typeof n.className==='string'&&n.className.trim()?'.'+n.className.trim().split(/\s+/).slice(0,3).join('.'):''))}return p.join(' > ')};
 const TEXT='p,span,small,strong,h1,h2,h3,h4,h5,label,a,li,td,th,button,option,summary,code,figcaption,legend';
 const seen=new Set<string>();
 const leaves:{el:Element;r:DOMRect;txt:string;path:string}[]=[];
 const all=Array.from(document.querySelectorAll(TEXT));
 /** Elemento escondido pelo recorte de um ancestral (details fechado, lista rolada,
  *  corpo colapsado): o retângulo existe no layout mas ninguém o enxerga. */
 const clipped=(el:Element,r:DOMRect):boolean=>{
  const back=document.querySelector('.modal-backdrop');
  if(back&&!back.contains(el))return true; // fica atrás do véu do modal: fora de vista
  if(el.closest('details')&&!el.closest('details')?.hasAttribute('open'))return true;
  let d=el.parentElement?el.parentElement.closest('details'):null;
  while(d){if(!d.hasAttribute('open'))return true;d=d.parentElement?d.parentElement.closest('details'):null}
  let n=el.parentElement;
  while(n&&n!==document.body){
   const cs=getComputedStyle(n);
   if(cs.display==='none'||cs.visibility==='hidden')return true;
   if(cs.overflowX!=='visible'||cs.overflowY!=='visible'){
    const pr=n.getBoundingClientRect();
    const w=Math.min(r.right,pr.right)-Math.max(r.left,pr.left);
    const h=Math.min(r.bottom,pr.bottom)-Math.max(r.top,pr.top);
    if(w<=1||h<=1)return true;
    if(w<r.width*0.5||h<r.height*0.5)return true;
   }
   n=n.parentElement;
  }
  return false;
 };
 for(const el of all){
  const r=el.getBoundingClientRect();
  if(r.width<2||r.height<2)continue;
  const cs=getComputedStyle(el);
  if(cs.visibility==='hidden'||cs.display==='none'||cs.opacity==='0')continue;
  const own=Array.from(el.childNodes).filter(n=>n.nodeType===3).map(n=>n.textContent||'').join('').trim();
  if(!own)continue;
  const key=path(el)+'|'+own.slice(0,40);
  if(seen.has(key))continue;
  if(clipped(el,r))continue;
  if(el.scrollWidth>el.clientWidth+1&&cs.overflowX==='hidden'){
   seen.add(key);out.push({k:'texto-cortado-h',sel:path(el),txt:own.slice(0,70),extra:`scrollW=${el.scrollWidth} clientW=${el.clientWidth}`});
  }
  if(el.scrollHeight>el.clientHeight+2&&cs.overflowY==='hidden'){
   seen.add(key);out.push({k:'texto-cortado-v',sel:path(el),txt:own.slice(0,70),extra:`scrollH=${el.scrollHeight} clientH=${el.clientHeight}`});
  }
  const par=el.parentElement;
  if(par&&el.tagName!=='DIV'){
   const pr=par.getBoundingClientRect();
   const pcs=getComputedStyle(par);
   if(pcs.position==='static'||pcs.position==='relative'){
    if(r.right>pr.right+1.5||r.left<pr.left-1.5){
     seen.add(key);out.push({k:'estoura-pai',sel:path(el),txt:own.slice(0,70),extra:`el[${Math.round(r.left)},${Math.round(r.right)}] pai[${Math.round(pr.left)},${Math.round(pr.right)}]`});
    }
   }
  }
  if(r.right>window.innerWidth+2){
   seen.add(key);out.push({k:'fora-da-tela',sel:path(el),txt:own.slice(0,70),extra:`right=${Math.round(r.right)} vw=${window.innerWidth}`});
  }
  leaves.push({el,r,txt:own,path:path(el)});
 }

 // 5. textos sobrepostos (descartando camadas posicionadas de propósito)
 const frags=(el:Element)=>Array.from(el.getClientRects());
 for(let i=0;i<leaves.length;i++)for(let j=i+1;j<leaves.length;j++){
  const a=leaves[i],b=leaves[j];
  if(a.el.contains(b.el)||b.el.contains(a.el))continue;
  const pa=getComputedStyle(a.el).position,pb=getComputedStyle(b.el).position;
  if(pa!=='static'||pb!=='static')continue;
  let bw=0,bh=0;
  for(const x of frags(a.el))for(const y of frags(b.el)){
   const w=Math.min(x.right,y.right)-Math.max(x.left,y.left);
   const h=Math.min(x.bottom,y.bottom)-Math.max(x.top,y.top);
   if(w>1&&h>1&&w*h>bw*bh){bw=w;bh=h}
  }
  if(bw>1&&bh>1){
   const key='overlap|'+a.path+a.txt.slice(0,20)+'|'+b.path+b.txt.slice(0,20);
   if(seen.has(key))continue;
   seen.add(key);out.push({k:'textos-sobrepostos',sel:a.path,txt:a.txt.slice(0,50)+'  ×  '+b.txt.slice(0,50),extra:`sobreposição ${Math.round(bw)}×${Math.round(bh)}px`});
  }
 }

 // 6. caixas irmãs do mesmo grid com alturas diferentes (visualmente torto)
 const boxes=Array.from(document.querySelectorAll('.card,.metric,.module,.preset-card,.profile-card'))
  .filter(el=>{const r=el.getBoundingClientRect();return r.width>10&&r.height>10});
 const groups=new Map<string,Element[]>();
 for(const el of boxes){
  const pr=el.parentElement;if(!pr)continue;
  if(getComputedStyle(pr).display!=='grid')continue;
  const key=String(Math.round(el.getBoundingClientRect().top))+'|'+String(pr.className);
  const g=groups.get(key)||[];g.push(el);groups.set(key,g);
 }
 for(const [,g] of groups){
  if(g.length<2)continue;
  const hs=g.map(el=>el.getBoundingClientRect().bottom);
  const min=Math.min(...hs),max=Math.max(...hs);
  if(max-min>1.5){
   const sel=g.map(el=>path(el)).join(' | ');
   const key='grid|'+sel;
   if(seen.has(key))continue;
   seen.add(key);out.push({k:'caixas-desalinhadas',sel:sel.slice(0,150),txt:'',extra:`bases variam ${Math.round(max-min)}px`});
  }
 }
 return out;
}

export function fmt(issues:Issue[],limit=20):string[] {
 const lines:string[]=[];
 for(const i of issues.slice(0,limit))lines.push(`  [${i.k}] ${i.sel}\n      txt="${i.txt}"  ${i.extra}`);
 if(issues.length>limit)lines.push(`  … +${issues.length-limit} demais`);
 return lines;
}
