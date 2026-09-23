import {invoke,isTauri} from '@tauri-apps/api/core';
import type {Profile,Flow,Note,Preset,Snapshot} from './types';
export const desktop=isTauri();
type Args=Record<string,unknown>;
type Preview={profiles:Profile[];flows:Flow[];notes:Record<string,Note[]>;presets:Preset[];theme:string;accent?:string};
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
 case 'community':result={balances:{},names:{},queue:[],songs:[],raffle:{open:false,keyword:'!sorteio',price:0,entries:{},winner:''},prediction:{open:false,title:'',options:[],bets:{},result:null},shop:[],redemptions:[],trivia:{}};break;
 default:throw Error('Abra o aplicativo desktop para usar esta função. Esta prévia permite editar perfis, comandos e notas; conexões e execução dependem do núcleo Rust.');
 }
 save(s);return result as T;
}
export function errorText(e:unknown){return e instanceof Error?e.message:String(e)}
