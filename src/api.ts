import {invoke,isTauri} from '@tauri-apps/api/core';
import type {Profile,Flow,Note,Preset,Snapshot} from './types';
export const desktop=isTauri();
type Args=Record<string,unknown>;
type Preview={profiles:Profile[];flows:Flow[];notes:Record<string,Note[]>;presets:Preset[];theme:string;accent?:string;discord?:Record<string,unknown>};
const initial:Preview={profiles:[],flows:[],notes:{},presets:[],theme:'dark'};
function read():Preview{try{return {...initial,...JSON.parse(localStorage.getItem('botlive-preview')||'{}')}}catch{return structuredClone(initial)}}
function save(s:Preview){localStorage.setItem('botlive-preview',JSON.stringify(s))}
export async function api<T=unknown>(op:string,args:Args={}):Promise<T>{
 if(desktop)return invoke<T>('rpc',{op,args});
 const s=read();const id=String(args.profileId||'');let result:unknown;
 switch(op){
 case 'snapshot':result={profiles:s.profiles,logs:[],statuses:{},theme:s.theme,accent:s.accent,apiPort:0,dataDir:'Prévia no navegador — dados separados do aplicativo desktop'} satisfies Snapshot;break;
 case 'profile.save':{const p=args.profile as Profile;if(!p.name.trim())throw Error('Dê um nome ao perfil.');s.profiles=s.profiles.filter(x=>x.id!==p.id).concat(p);result=p;break}
 case 'profile.delete':s.profiles=s.profiles.filter(p=>p.id!==id);s.flows=s.flows.filter(f=>f.profileId!==id);delete s.notes[id];break;
 case 'command.counters':result={};break;
 case 'flows':result=s.flows.filter(f=>f.profileId===id);break;
 case 'flow.save':{const f=args.flow as Flow;s.flows=s.flows.filter(x=>x.id!==f.id).concat(f);result=f;break}
 case 'flow.delete':s.flows=s.flows.filter(f=>f.id!==args.id||f.profileId!==id);break;
 case 'logs':result=[];break;
 case 'stats':result={messages:0,actions:0,followers:0,hours:[],commands:[]};break;
 case 'notes':result=s.notes[id]||[];break;
 case 'note.save':{s.notes[id]=(s.notes[id]||[]).filter(n=>n.path!==args.path).concat({path:String(args.path),content:String(args.content)});break}
 case 'note.delete':s.notes[id]=(s.notes[id]||[]).filter(n=>n.path!==args.path);break;
 case 'presets':result=s.presets;break;
 case 'preset.save':{const p=args.preset as Preset;s.presets=s.presets.filter(x=>x.id!==p.id).concat(p);break}
 case 'settings':if(args.key==='theme')s.theme=String(args.value);if(args.key==='accent')s.accent=String(args.value);break;
 case 'settings.get':result={theme:s.theme,apiPort:9876,apiToken:'Disponível no aplicativo desktop',updateEndpoint:'',updatePublicKey:'',autoUpdate:false};break;
 case 'secret.status':result={ai:false,bot:false,channel:false};break;
  case 'discord.status':{const c=s.discord||{};result={status:'offline',running:false,enabled:Boolean(c.enabled),hasToken:false,config:c,guilds:[{id:'123456789012345678',name:'Servidor da comunidade'}],guildId:String(c.guildId||''),channels:[{id:'111111111111111111',name:'geral',type:0},{id:'222222222222222222',name:'avisos',type:0},{id:'333333333333333333',name:'sorteios',type:0}],roles:[{id:'444444444444444444',name:'Moderador'},{id:'555555555555555555',name:'Assinante'}],moduleEnabled:false,platform:'twitch',auditCount:0};break}
  case 'discord.save':s.discord=(args.config||{})as Record<string,unknown>;break;
  case 'discord.discover':result={guilds:[{id:'123456789012345678',name:'Servidor da comunidade'}],guild:{id:'123456789012345678',name:'Servidor da comunidade',member_count:42},channels:[{id:'111111111111111111',name:'geral',type:0},{id:'222222222222222222',name:'avisos',type:0},{id:'333333333333333333',name:'sorteios',type:0}],roles:[{id:'444444444444444444',name:'Moderador'},{id:'555555555555555555',name:'Assinante'}],identity:{id:'1',name:'BotLive',avatar:''}};break;
  case 'discord.connect':result={identity:{id:'1',name:'BotLive'},status:'connecting'};break;
  case 'discord.disconnect':break;
  case 'discord.action':{const a=String(args.action||'');if(a==='members')result=[{user:{id:'999999999999999999',username:'apoiador',global_name:'Apoiador'},nick:null}];else if(a==='linkCreate')result={code:'482913',hint:'Código gerado.'};else if(a==='linkList')result={links:[{discordId:'999999999999999999',twitchId:'482115'}],pending:{}};else if(a==='xpRank')result=[{id:'999999999999999999',name:'Apoiador',xp:120,level:1,next:400}];else if(a==='auditExport')result='acao,alvo,plataformas,motivo\\nwarn,Apoiador,discord+twitch,Prova de auditoria\\n';else if(a==='audit')result=[{id:'a1',action:'warn',target:'999999999999999999',targetName:'Apoiador',reason:'Prova de auditoria',platforms:['discord','twitch'],undoable:true,at:'25/09/2026 10:00'}];else if(a==='warnlist')result=[{target:'999999999999999999',targetName:'Apoiador',reason:'Prova de auditoria',date:'25/09/2026'}];else if(a==='birthdayList')result=[{user:'999999999999999999',name:'Apoiador',day:25,month:9,year:1990}];else if(a==='giveawayList')result=[{id:'g1',prize:'Nitro',status:'open',entries:['999999999999999999']}];else result=null;break}
  case 'secret.save':break;
 case 'community':result={balances:{},names:{},queue:[],songs:[],raffle:{open:false,keyword:'!sorteio',price:0,entries:{},winner:''},prediction:{open:false,title:'',options:[],bets:{},result:null},shop:[],redemptions:[],trivia:{}};break;
 default:throw Error('Abra o aplicativo desktop para usar esta função. Esta prévia permite editar perfis, comandos e notas; conexões e execução dependem do núcleo Rust.');
 }
 save(s);return result as T;
}
export function errorText(e:unknown){return e instanceof Error?e.message:String(e)}
