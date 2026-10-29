import {useCallback,useEffect,useState,useRef} from 'react';
import {RefreshCw,Users,X} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card} from './components';

type Viewers={total:number;users:string[]};

/** Lista do chat em tempo real: uma fonte para o indicador do topo e para o
 *  card da Visão geral. Atualiza sozinho a cada 30 s; some quando a live está
 *  fora do ar (quem não está ao vivo não precisa ver essa lista). */
export function useChatViewers(profileId:string,online:boolean){
 const [viewers,setViewers]=useState<Viewers|null>(null);
 const [error,setError]=useState('');
 const generation=useRef(0);
 const refresh=useCallback(async()=>{
  const gen=generation.current;
  try{const data=await api<Viewers>('labels.viewers',{profileId});if(gen!==generation.current)return;if(!data||!Array.isArray(data.users)||typeof data.total!=='number')throw Error('Presença indisponível');setViewers(data);setError('')}
  catch(e){if(gen===generation.current)setError(errorText(e))}
 },[profileId]);
 useEffect(()=>{
  generation.current++;setViewers(null);setError('');
  if(!desktop||!online)return;
  void refresh();const timer=setInterval(()=>{void refresh()},30000);
  return()=>{generation.current++;clearInterval(timer)};
 },[refresh,online]);
 return {viewers,error,refresh};
}

/**
 * Indicador de presença no topo do app: enquanto o perfil está conectado,
 * mostra quantas pessoas estão no chat e, ao clicar, a lista inteira.
 * A lista se atualiza sozinha a cada 30 s (o backend já mantém o cache).
 */
export default function ChatPresence({profileId,online}:{profileId:string;online:boolean}){
 const [open,setOpen]=useState(false);
 const {viewers,error,refresh}=useChatViewers(profileId,online);
 if(!desktop||!online)return null;
 const total=viewers?.total;
 return <div className="presence">
  <button className="presence-pill" aria-expanded={open} aria-label="Quem está no chat agora" onClick={()=>setOpen(v=>!v)}>
   <Users size={15}/><span>{typeof total==='number'?`${total} no chat`:'Presença indisponível'}</span>
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

/** Card da Visão geral: quem está no chat agora, com números e nomes, em
 *  tempo real. Some quando o streamer não está ao vivo. */
export function ChatNow({profileId,online}:{profileId:string;online:boolean}){
 const {viewers,error,refresh}=useChatViewers(profileId,online);
 if(!desktop||!online)return null;
 const total=viewers?.total;
 const users=viewers?.users||[];
 return <Card title="Quem está no chat agora" action={<button onClick={()=>void refresh()}><RefreshCw size={14}/>Atualizar</button>}>
  <div className="stat-inline"><strong>{typeof total==='number'?total:'—'}</strong><span>com o chat aberto{total===1?"":" mesmo sem falar"}</span></div>
  {error&&<p role="alert" className="inline-error">{error}</p>}
  <div className="message-readable">{users.length
   ?users.map(u=><span className="variable-chip" key={u}>{u}</span>)
   :<p className="help">A lista detalhada considera as primeiras 1000 pessoas retornadas pela Twitch.</p>}</div>
  <p className="help">Presença no chat, não espectadores assistindo. Atualiza a cada 30 segundos enquanto o bot está conectado.</p>
 </Card>;
}
