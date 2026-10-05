import {useCallback,useEffect,useState} from 'react';
import {open} from '@tauri-apps/plugin-dialog';
import {Play,Plus,Search,Trash2,Upload} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Empty} from './components';
type Asset={id:string;kind:string;name:string};
export default function Sounds({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [assets,setAssets]=useState<Asset[]>([]);
 const [query,setQuery]=useState('');
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const [playing,setPlaying]=useState('');
 const load=useCallback(async()=>{
  try{const v=await api<{assets:Asset[]}>('chatExtras.get',{profileId});setAssets(v.assets.filter(a=>a.kind==='sound'))}
  catch(e){setError(errorText(e))}
 },[profileId]);
 useEffect(()=>{if(desktop)void load()},[load]);
 async function upload(){
  setBusy(true);setError('');
  try{
   const path=await open({multiple:false,filters:[{name:'Áudio',extensions:['wav','mp3','ogg']}]});
   if(typeof path!=='string')return;
   const asset=await api<Asset>('chatExtras.import',{profileId,kind:'sound',path});
   setAssets(a=>a.concat(asset));notify('Som adicionado à biblioteca.');
  }catch(e){setError(errorText(e))}finally{setBusy(false)}
 }
 async function play(a:Asset){
  setError('');setPlaying(a.id);
  try{const url=await api<string>('chatExtras.audio',{profileId,asset:a.id});await new Audio(url).play()}
  catch(e){setError(errorText(e))}finally{setPlaying('')}
 }
 async function remove(a:Asset){
  if(!confirm(`Apagar “${a.name}” da biblioteca? Comandos e pessoas que usam este som ficam sem áudio.`))return;
  setBusy(true);setError('');
  try{await api('chatExtras.remove',{profileId,asset:a.id});setAssets(list=>list.filter(x=>x.id!==a.id));notify('Som apagado.')}
  catch(e){setError(errorText(e))}finally{setBusy(false)}
 }
 const visible=assets.filter(a=>(a.name+' '+a.id).toLowerCase().includes(query.toLowerCase()));
 return <>
 <div className="section-tools"><p className="help">{assets.length} sons · usados em comandos, timers, automações e sons por pessoa</p><button className="primary" disabled={!desktop||busy} onClick={()=>void upload()}><Plus size={16}/>Adicionar som</button></div>
 {!desktop&&<p className="notice">A biblioteca de sons fica no aplicativo desktop.</p>}
 {error&&<p role="alert" className="inline-error">{error}</p>}
 {visible.length?<Card><div className="table-scroll"><table><thead><tr><th>Som</th><th></th></tr></thead><tbody>{visible.map(a=><tr key={a.id}><td><strong>{a.name}</strong></td><td><div className="row"><button disabled={!desktop||!!playing} onClick={()=>void play(a)}><Play size={15}/>{playing===a.id?'Tocando…':'Ouvir'}</button><button className="icon-button danger" aria-label={'Apagar '+a.name} disabled={!desktop||busy} onClick={()=>void remove(a)}><Trash2 size={16}/></button></div></td></tr>)}</tbody></table></div></Card>:<Empty title={query?'Nenhum som com esse nome':'Sua biblioteca de sons'} action={<button className="primary" disabled={!desktop||busy} onClick={()=>void upload()}><Upload size={16}/>Adicionar primeiro som</button>}>{query?'Tente outro nome.':'WAV, MP3 ou OGG até 5 MiB. Depois escolha em comandos, timers, automações e sons por pessoa.'}</Empty>}
 <Card title="Onde usar"><p className="help">Comandos, timers e automações têm <strong>Tocar áudio ao disparar</strong>; Respostas e sons tem <strong>Sons por espectador</strong>. O som toca na saída do BotLive — capture essa saída no OBS para a live ouvir.</p><div className="search"><Search size={17}/><input aria-label="Buscar sons" placeholder="Buscar sons…" value={query} onChange={e=>setQuery(e.target.value)}/></div></Card>
 </>;
}
