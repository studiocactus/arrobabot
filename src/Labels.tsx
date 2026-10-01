import {useCallback,useEffect,useState} from 'react';
import {open} from '@tauri-apps/plugin-dialog';
import {FolderOpen,RefreshCw,Save,Users} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Empty,Field,Toggle} from './components';
type Def={key:string;file:string;name:string;tokens:string;enabled:boolean;template:string;value:string};
type Data={config:{enabled:boolean;folder:string};defs:Def[];viewers:string[];viewersTotal:number};
type Viewers={total:number;users:string[]};
export default function Labels({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [data,setData]=useState<Data|null>(null);
 const [viewers,setViewers]=useState<Viewers|null>(null);
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const load=useCallback(async()=>{
  try{setData(await api<Data>('labels.get',{profileId}))}
  catch(e){setError(errorText(e))}
 },[profileId]);
 useEffect(()=>{void load()},[load]);
 async function run(fn:()=>Promise<void>){setBusy(true);setError('');try{await fn()}catch(e){setError(errorText(e))}finally{setBusy(false)}}
 async function pickFolder(){
  const path=await open({directory:true,multiple:false});
  if(typeof path!=='string')return;
  await run(async()=>{await api('labels.folder',{profileId,path});await load();notify('Pasta dos rótulos salva.')});
 }
 async function save(){
  if(!data)return;
  const labels:Record<string,{enabled:boolean;template:string}>={};
  data.defs.forEach(d=>{labels[d.key]={enabled:d.enabled,template:d.template}});
  await run(async()=>{await api('labels.save',{profileId,config:{enabled:data.config.enabled,folder:data.config.folder,labels}});await load();notify('Rótulos salvos e arquivos gravados.')});
 }
 async function refreshViewers(){await run(async()=>{setViewers(await api<Viewers>('labels.viewers',{profileId}))})}
 async function refreshNow(){await run(async()=>{await api('labels.refresh',{profileId});await load();notify('Totais atualizados da Twitch.')})}
 const setDef=(key:string,patch:Partial<Def>)=>setData(d=>d?{...d,defs:d.defs.map(x=>x.key===key?{...x,...patch}:x)}:d);
 if(error&&!data)return <Empty title="Rótulos indisponíveis">{error}</Empty>;
 if(!data)return <Empty title="Carregando rótulos">Aguarde a leitura da configuração.</Empty>;
 return <div>
 <Card title="Arquivos .txt para o OBS">
  <div className="list-row"><div><strong>Rótulos ativos</strong><small>Só na Twitch e com o perfil conectado. Os arquivos saem na pasta abaixo.</small></div><Toggle label="Rótulos ativos" checked={data.config.enabled} onChange={v=>setData({...data,config:{...data.config,enabled:v}})}/></div>
  <Field label="Pasta dos arquivos" hint="Escolha onde o OBS vai ler os .txt. Cada rótulo vira um arquivo fixo."><input value={data.config.folder} placeholder="C:/Live/rotulos" onChange={e=>setData({...data,config:{...data.config,folder:e.target.value}})}/></Field>
  <div className="row">
   <button disabled={!desktop||busy} onClick={()=>void pickFolder()}><FolderOpen size={15}/>Escolher pasta</button>
   <button disabled={!desktop||busy} onClick={()=>void refreshNow()}><RefreshCw size={15}/>Atualizar agora</button>
   <button className="primary" disabled={!desktop||busy} onClick={()=>void save()}><Save size={15}/>Salvar rótulos</button>
  </div>
  {!desktop&&<p className="help">A pasta e a gravação funcionam no aplicativo desktop.</p>}
  {error&&<p role="alert" className="inline-error">{error}</p>}
 </Card>
 <Card title="Rótulos e modelos">
  <p className="help">Use os tokens de cada linha no modelo. O valor mostra o que sai no .txt agora.</p>
  {data.defs.map(d=><div className="list-row" key={d.key}><div style={{flex:1,minWidth:0}}><strong>{d.name}</strong><small>{d.file} · tokens: {d.tokens}</small><input aria-label={'Modelo de '+d.name} value={d.template} maxLength={200} onChange={e=>setDef(d.key,{template:e.target.value})}/><small>Agora: {d.value||'—'}</small></div><Toggle label={'Ativar '+d.name} checked={d.enabled} onChange={v=>setDef(d.key,{enabled:v})}/></div>)}
 </Card>
 <Card title="Quem está no chat agora" action={<button disabled={!desktop||busy} onClick={()=>void refreshViewers()}><Users size={15}/>Atualizar</button>}>
  <p className="help">Quem está com o chat aberto, mesmo sem falar. Exige o bot moderador e a conta do bot autorizada de novo (leitura de chatters). O próprio bot não entra na conta de horas.</p>
  {viewers?<><p><strong>{viewers.total} no chat</strong></p>{viewers.users.length?<div className="viewer-list">{viewers.users.map(u=><span key={u}>{u}</span>)}</div>:<p className="help">Ninguém além do bot.</p>}</>:<p className="help">Clique em Atualizar para ver a lista.</p>}
 </Card>
 </div>;
}
