import {useEffect,useState} from 'react';
import {api,desktop,errorText} from './api';
import {Card} from './components';
import {Headphones,Mic,Monitor,RefreshCw} from 'lucide-react';
type Status={channel:string;audio:{clients:number};voice:{phase:string;restorePending:boolean;testActive:boolean};voiceQueue:{activeName:string|null;waiting:number;pendingRestore:boolean};obs:{temporary:number;pendingRestore:number};running:number;issues:number};
export type ConsoleAttention={text:string;page:string;liveOnly?:boolean};
export default function LiveConsole({profileId,onGo,onAttention}:{profileId:string;onGo:(page:string)=>void;onAttention?:(items:ConsoleAttention[])=>void}){
 const [status,setStatus]=useState<Status|null>(null),[error,setError]=useState(''),[at,setAt]=useState(''),[retry,setRetry]=useState(0);
 useEffect(()=>{if(!desktop)return;let stopped=false;let timer:ReturnType<typeof setTimeout>;
  async function tick(){try{const data=await api<Status>('liveConsole.get',{profileId});if(!data||!data.audio||!data.voice||!data.obs||!data.voiceQueue)throw Error('Não foi possível ler o estado operacional');if(!stopped){setStatus(data);setError('');setAt(new Date().toLocaleTimeString('pt-BR'))}}catch(e){if(!stopped)setError(errorText(e))}finally{if(!stopped)timer=setTimeout(tick,5000)}}
  void tick();return()=>{stopped=true;clearTimeout(timer)};
 },[profileId,retry]);
 useEffect(()=>{const items:ConsoleAttention[]=[];if(error)items.push({text:'Não foi possível atualizar o estado de áudio e execuções.',page:'history'});if(status&&status.audio.clients===0)items.push({text:'Fonte de áudio do OBS desconectada; sons do bot usam a saída local.',page:'audio',liveOnly:true});if(status?.voice.restorePending||status?.voiceQueue.pendingRestore)items.push({text:'Voicemod: a restauração da voz precisa de atenção.',page:'vozfx'});if(status?.obs.pendingRestore)items.push({text:`OBS: ${status.obs.pendingRestore} restauração(ões) pendente(s).`,page:'obs'});onAttention?.(items)},[status,error,onAttention]);
 const voicePending=!!status&&(status.voice.restorePending||status.voiceQueue.pendingRestore);
 return <Card className="live-console" title="Operação e áudio" action={<button onClick={()=>setRetry(n=>n+1)}>Atualizar estado</button>}>
  {!desktop?<p className="help">O estado real dos serviços aparece no aplicativo desktop.</p>:<>
   {error&&<p role="alert" className="inline-error">{error}. {status?'Os dados anteriores podem estar desatualizados.':''}</p>}
   {!status&&!error&&<p role="status" className="help">Consultando os serviços…</p>}
   {status&&<div className={'live-console-grid'+(error?' is-stale':'')}>
    <div className="operation-card"><header><span className="operation-icon"><Headphones size={20}/></span><strong>Sons do bot</strong><span className={'operation-tag '+(status.audio.clients>0?'is-ready':'')}>{status.audio.clients>0?'Fonte conectada':'Saída local'}</span></header><div className="operation-value">{status.audio.clients}<small>{status.audio.clients===1?'fonte de áudio no OBS':'fontes de áudio no OBS'}</small></div><p className="help">{status.audio.clients>0?'Confira a reprodução na transmissão. A conexão sozinha não confirma som audível.':'Sem fonte OBS; os sons usam a saída local do aplicativo.'}</p><footer><button onClick={()=>onGo('audio')}>Conferir saída de áudio</button></footer></div>
    <div className="operation-card"><header><span className="operation-icon"><Mic size={20}/></span><strong>Voicemod da máquina</strong><span className={'operation-tag '+(voicePending?'is-warning':status.voice.phase==='connected'?'is-ready':'')}>{voicePending?'Atenção':status.voice.phase==='connected'?'Conectado':'Não conectado'}</span></header><p className="operation-description">{voicePending?'Restauração da voz pendente':status.voiceQueue.activeName|| (status.voice.testActive?'Teste em andamento':status.voice.phase==='connected'?'Pronto para efeitos de voz':'Abra o Voicemod para conectar')}</p><div className="operation-facts"><span><b>{status.voiceQueue.waiting}</b> na fila</span><span>{status.voiceQueue.activeName?'Efeito ativo':'Sem efeito na fila ativa'}</span></div><footer><button onClick={()=>onGo('vozfx')}>{voicePending?'Recuperar na tela Voicemod':'Conferir voz e fila'}</button></footer></div>
    <div className="operation-card"><header><span className="operation-icon"><Monitor size={20}/></span><strong>OBS e execuções</strong><span className={'operation-tag '+(status.obs.pendingRestore?'is-warning':'')}>{status.obs.pendingRestore?'Restauração pendente':'Monitorando ações'}</span></header><div className="operation-value">{status.running}<small>{status.running===1?'execução em andamento':'execuções em andamento'}</small></div><div className="operation-facts"><span><b>{status.obs.temporary}</b> efeitos temporários</span><span><b>{status.obs.pendingRestore}</b> restaurações pendentes</span></div><p className="help">{status.issues} {status.issues===1?'falha ou interrupção registrada':'falhas ou interrupções registradas'} no histórico. Isso não representa, por si só, um problema atual.</p><footer><button onClick={()=>onGo('history')}>Investigar execuções</button><button onClick={()=>onGo('obs')}>Conferir OBS</button></footer></div>
   </div>}
   {at&&<p className="help operation-updated"><RefreshCw size={12}/>Última leitura: {at} · atualização a cada 5 segundos</p>}
  </>}
 </Card>;
}
