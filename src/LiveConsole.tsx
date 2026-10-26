import {useEffect,useState} from 'react';
import {api,desktop,errorText} from './api';
import {Card} from './components';
type Status={channel:string;audio:{clients:number};voice:{phase:string;restorePending:boolean;testActive:boolean};voiceQueue:{activeName:string|null;waiting:number;pendingRestore:boolean};obs:{temporary:number;pendingRestore:number};running:number;issues:number};
export default function LiveConsole({profileId,onGo}:{profileId:string;onGo:(page:string)=>void}){
 const [status,setStatus]=useState<Status|null>(null),[error,setError]=useState(''),[at,setAt]=useState(''),[retry,setRetry]=useState(0);
 useEffect(()=>{if(!desktop)return;let stopped=false;let timer:ReturnType<typeof setTimeout>;
  async function tick(){try{const data=await api<Status>('liveConsole.get',{profileId});if(!data||!data.audio||!data.voice||!data.obs||!data.voiceQueue)throw Error('Não foi possível ler o estado operacional');if(!stopped){setStatus(data);setError('');setAt(new Date().toLocaleTimeString('pt-BR'))}}catch(e){if(!stopped)setError(errorText(e))}finally{if(!stopped)timer=setTimeout(tick,5000)}}
  void tick();return()=>{stopped=true;clearTimeout(timer)};
 },[profileId,retry]);
 const voicePending=!!status&&(status.voice.restorePending||status.voiceQueue.pendingRestore);
 return <Card className="live-console" title="Central da live" action={<button onClick={()=>setRetry(n=>n+1)}>Atualizar estado</button>}>
  {!desktop?<p className="help">O estado real dos serviços aparece no aplicativo desktop.</p>:<>
   {error&&<p role="alert" className="inline-error">{error}. {status?'Os dados anteriores podem estar desatualizados.':''}</p>}
   {!status&&!error&&<p role="status" className="help">Consultando os serviços…</p>}
   {status&&<div className="live-console-grid">
    <div><strong>Canal</strong><p>{status.channel==='online'?'Conectado':status.channel==='offline'?'Desconectado':'Em transição: '+status.channel}</p><button onClick={()=>onGo('profiles')}>Conferir contas</button></div>
    <div><strong>Sons do bot</strong><p>{status.audio.clients>0?`${status.audio.clients} fonte(s) conectada(s)`:'Nenhuma fonte de áudio conectada'}</p><small>Conexão não comprova som audível.</small><button onClick={()=>onGo('audio')}>Conferir saída de áudio</button></div>
    <div><strong>Voicemod</strong><p>{voicePending?'Restauração precisa de atenção':status.voiceQueue.activeName||status.voice.testActive?'Efeito ou teste em andamento':status.voice.phase==='connected'?'Conectado':'Não conectado'}</p><small>{status.voiceQueue.waiting} pedido(s) aguardando</small><button onClick={()=>onGo('vozfx')}>{voicePending?'Recuperar na tela Voicemod':'Conferir voz e fila'}</button></div>
    <div><strong>OBS e execuções</strong><p>{status.running} execução(ões) em andamento · {status.issues} falha(s) ou interrupção(ões) recentes</p><small>{status.obs.temporary} temporário(s) no OBS · {status.obs.pendingRestore} restauração(ões) pendente(s)</small><div className="row"><button onClick={()=>onGo('history')}>Investigar execuções</button><button onClick={()=>onGo('obs')}>Conferir OBS</button></div></div>
   </div>}
   {at&&<p className="help">Última leitura: {at}. Consulta a cada 5 segundos; nenhum serviço é conectado ou repetido por esta tela.</p>}
  </>}
 </Card>;
}
