import {useCallback,useEffect,useState} from 'react';
import {RefreshCw,Users,X} from 'lucide-react';
import {api,desktop,errorText} from './api';

type Viewers={total:number;users:string[]};

/**
 * Indicador de presença no topo do app: enquanto o perfil está conectado,
 * mostra quantas pessoas estão no chat e, ao clicar, a lista inteira.
 * A lista se atualiza sozinha a cada 30 s (o backend já mantém o cache).
 */
export default function ChatPresence({profileId,online}:{profileId:string;online:boolean}){
 const [viewers,setViewers]=useState<Viewers|null>(null);
 const [open,setOpen]=useState(false);
 const [error,setError]=useState('');
 const refresh=useCallback(async()=>{
  try{setViewers(await api<Viewers>('labels.viewers',{profileId}));setError('')}
  catch(e){setError(errorText(e))}
 },[profileId]);
 useEffect(()=>{
  if(!desktop||!online){setViewers(null);return}
  void refresh();
  const timer=setInterval(()=>{void refresh()},30000);
  return()=>clearInterval(timer);
 },[refresh,online]);
 if(!desktop||!online)return null;
 const total=viewers?.total||0;
 return <div className="presence">
  <button className="presence-pill" aria-expanded={open} aria-label="Quem está no chat agora" onClick={()=>setOpen(v=>!v)}>
   <Users size={15}/><span>{total>0?`${total} no chat`:'Chat ao vivo'}</span>
  </button>
  {open&&<div className="presence-panel" role="dialog" aria-label="Quem está no chat agora">
   <header><strong>Quem está no chat</strong><button className="icon-button" aria-label="Fechar lista do chat" onClick={()=>setOpen(false)}><X size={15}/></button></header>
   {error&&<p role="alert" className="inline-error">{error}</p>}
   {viewers
    ?(viewers.users.length
      ?<ul>{viewers.users.map(u=><li key={u}>{u}</li>)}</ul>
      :<p className="help">Ninguém falou no chat por enquanto.</p>)
    :<p className="help">Lendo a lista do chat…</p>}
   <footer><button onClick={()=>void refresh()}><RefreshCw size={14}/>Atualizar agora</button></footer>
  </div>}
 </div>;
}
