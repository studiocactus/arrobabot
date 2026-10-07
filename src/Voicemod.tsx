import {useCallback,useEffect,useMemo,useState} from 'react';
import {Link2,Link2Off,Play,RefreshCw,Save,Search,ShieldAlert,Square,Undo2} from 'lucide-react';
import {listen} from '@tauri-apps/api/event';
import {api,desktop,errorText} from './api';
import {Card,Field} from './components';

type Voice={id:string;friendlyName:string;enabled?:boolean};
type Status={phase:string;message:string;port:number;voices:Voice[];currentVoice:string;currentName:string;voiceChanger:boolean|null;hearMyself:boolean|null;license:string;attempts:number};
type Test={active:boolean;phase:string;voiceId:string;voiceName:string;seconds:number;remainingMs:number;interrupted:boolean;manual:boolean};
type Outcome={kind:string;detail:string};
type Snap={status:Status;test:Test;outcome:Outcome;hasKey:boolean;defaultSecs:number;minSecs:number;maxSecs:number};

const PHASE:{[k:string]:{cls:string;text:string}}={
 disconnected:{cls:'status-pill off',text:'Desconectado'},
 searching:{cls:'status-pill wait',text:'Procurando Voicemod'},
 authorizing:{cls:'status-pill wait',text:'Autorizando'},
 connected:{cls:'status-pill on',text:'Conectado'},
 failed:{cls:'status-pill off',text:'Falha'}
};

const BLANK:Snap={
 status:{phase:'disconnected',message:'Prévia no navegador: a conexão local só existe no aplicativo desktop.',port:0,voices:[],currentVoice:'',currentName:'',voiceChanger:null,hearMyself:null,license:'',attempts:0},
 test:{active:false,phase:'',voiceId:'',voiceName:'',seconds:0,remainingMs:0,interrupted:false,manual:false},
 outcome:{kind:'',detail:''},hasKey:false,defaultSecs:10,minSecs:1,maxSecs:60
};

function tone(kind:string){
 if(kind==='restored')return 'ok';
 if(kind==='manual')return 'info';
 if(kind==='restoring')return 'warn';
 if(kind)return 'bad';
 return '';
}

export default function Voicemod({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [snap,setSnap]=useState<Snap|null>(null);
 const [keyInput,setKeyInput]=useState('');
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const [query,setQuery]=useState('');
 const [selected,setSelected]=useState('');
 const [secs,setSecs]=useState<number|null>(null);

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{setSnap(await api<Snap>('voicemod.get',{profileId}))}catch(e){setError(errorText(e))}
 },[profileId]);
 useEffect(()=>{if(desktop)void load()},[load]);
 useEffect(()=>{
  if(!desktop)return;
  let alive=true;const off:(()=>void)[]=[];
  listen<Partial<Snap>>('voicemod',e=>{if(alive)setSnap(s=>s?{...s,...e.payload}:s)}).then(f=>{if(alive)off.push(f);else f()}).catch(()=>{});
  return()=>{alive=false;off.forEach(f=>f())};
 },[]);

 async function run(fn:()=>Promise<void>){
  setBusy(true);setError('');
  try{await fn()}catch(e){setError(errorText(e))}
  // Volta a ler o estado depois de agir (o backend também publica o evento):
  // assim a tela mostra Falha, teste ativo ou restauração sem depender do tempo.
  try{setSnap(await api<Snap>('voicemod.get',{profileId}))}catch{}
  setBusy(false);
 }
 const saveKey=()=>run(async()=>{
  await api('voicemod.key',{profileId,value:keyInput});setKeyInput('');await load();notify('Chave da Control API guardada no cofre.');
 });
 const connect=()=>run(async()=>{await api('voicemod.connect',{profileId});notify('Voicemod verificado.');});
 const disconnect=()=>run(async()=>{
  const v=await api<{stoppedTest:boolean;restored:boolean;detail:string}>('voicemod.disconnect',{profileId});
  await load();notify(v.stoppedTest?(v.restored?'Teste encerrado e voz restaurada.':'Teste encerrado. Confira o Histórico.'): 'Desconectado do Voicemod.');
 });
 const refresh=()=>run(async()=>{await api('voicemod.refresh',{profileId});});
 const startTest=()=>run(async()=>{
  await api('voicemod.testStart',{profileId,voiceId:selected,seconds:secs??undefined});notify('Teste iniciado. O microfone muda até o fim do teste.');
 });
 const stopTest=()=>run(async()=>{await api('voicemod.testStop',{profileId});notify('Teste encerrado.');});
 const recover=()=>run(async()=>{await api('voicemod.recover',{profileId});notify('Estado anterior restaurado.');});

 const s=snap||BLANK;
 const phase=PHASE[s.status.phase]||PHASE.disconnected;
 const connected=s.status.phase==='connected';
 const pending=s.outcome.kind==='unconfirmed'||s.outcome.kind==='restoring';
 const filtered=useMemo(()=>{
  const q=query.trim().toLowerCase();
  return s.status.voices.filter(v=>!q||v.friendlyName.toLowerCase().includes(q)||v.id.toLowerCase().includes(q));
 },[s.status.voices,query]);
 const durations=[5,10,15,30,60].filter(n=>n>=s.minSecs&&n<=s.maxSecs);
 const wanted=secs??s.defaultSecs;
 const duration=durations.includes(wanted)?wanted:(durations.find(n=>n>=wanted)??durations[durations.length-1]??10);
 const left=Math.max(0,Math.ceil(s.test.remainingMs/1000));
 const changer=s.status.voiceChanger===null?'—':(s.status.voiceChanger?'Ligado':'Desligado');
 const hear=s.status.hearMyself===null?'—':(s.status.hearMyself?'Ligado':'Desligado');
 const tag=(v:Voice)=>v.id===s.status.currentVoice?'atual':(v.enabled===false?'bloqueada':'');

 return <div>
 <Card title="Voicemod nesta máquina">
  <div className="list-row"><div><strong>Estado</strong><small>{s.status.message}</small></div><span className={phase.cls}><span className="status-dot" aria-hidden="true"></span><span>{phase.text}</span></span></div>
  <div className="row">
   {!connected
    ?<button className="primary" disabled={!desktop||busy} onClick={()=>void connect()}><Link2 size={15}/>Conectar</button>
    :<button disabled={!desktop||busy} onClick={()=>void disconnect()}><Link2Off size={15}/>Desconectar</button>}
   <button disabled={!desktop||busy||!connected} onClick={()=>void refresh()}><RefreshCw size={15}/>Atualizar vozes</button>
  </div>
  {error&&<p role="alert" className="inline-error">{error}</p>}
  {!desktop&&<p className="help">Prévia no navegador: conectar, listar vozes e testar exigem o aplicativo desktop. Aqui a tela aparece apenas para conferência.</p>}
 </Card>

 <Card title="Chave da Control API">
  <p className="help">A Voicemod entrega a chave pelo formulário oficial em <a className="link" href="https://control-api.voicemod.net/getting-started/" target="_blank" rel="noreferrer">control-api.voicemod.net/getting-started</a>. A chave fica só no cofre do sistema: nunca entra em log, preset, exportação ou código.</p>
  <Field label="Chave" hint={s.hasKey?'Chave guardada no cofre. Preencha para trocar.':'Nenhuma chave guardada neste perfil.'}>
   <input type="password" autoComplete="new-password" value={keyInput} placeholder={s.hasKey?'••••••••':'Cole a chave recebida'} onChange={e=>setKeyInput(e.target.value)}/>
  </Field>
  <div className="row"><button className="primary" disabled={!desktop||busy||!keyInput.trim()} onClick={()=>void saveKey()}><Save size={15}/>Salvar chave</button></div>
 </Card>

 <Card title="Vozes do Voicemod" action={<span className="list-count">{filtered.length} de {s.status.voices.length}</span>}>
  <div className="list-row"><div><strong>Voz atual</strong><small>{s.status.currentName?(s.status.currentName+' · '+s.status.currentVoice):'—'}</small></div></div>
  <div className="list-row"><div><strong>Modificador de voz</strong><small>Ligado ou desligado dentro do Voicemod</small></div><span className={'status-pill '+(s.status.voiceChanger?'on':'off')}><span className="status-dot" aria-hidden="true"></span><span>{changer}</span></span></div>
  <div className="list-row"><div><strong>Ouvir minha voz</strong><small>Lido da API, nunca alterado pelo BotLive</small></div><span className="status-pill off"><span className="status-dot" aria-hidden="true"></span><span>{hear}</span></span></div>
  <div className="list-row"><div><strong>Licença</strong><small>Decide quais vozes aparecem liberadas</small></div><span className="status-pill off"><span className="status-dot" aria-hidden="true"></span><span>{s.status.license||'—'}</span></span></div>
  <div className="section-tools">
   <div className="search"><Search size={17}/><input aria-label="Buscar voz pelo nome" placeholder="Buscar voz pelo nome…" value={query} onChange={e=>setQuery(e.target.value)}/></div>
  </div>
  <div className="voice-list">
   {filtered.map(v=><button key={v.id} type="button" aria-pressed={selected===v.id} className={'voice-row'+(selected===v.id?' selected':'')} disabled={!desktop||busy||v.enabled===false} onClick={()=>setSelected(v.id)} title={v.enabled===false?'Voz não liberada pela licença':v.friendlyName}>
    <strong>{v.friendlyName||v.id}</strong>
    <span className="voice-id">{v.id}</span>
    {tag(v)&&<span className="voice-tag">{tag(v)}</span>}
   </button>)}
   {!filtered.length&&<p className="voice-empty">{desktop?(s.status.voices.length?'Nenhuma voz com esse nome.':'Ainda sem vozes. Conecte e use Atualizar vozes.'):'Prévia no navegador: a lista de vozes só existe no aplicativo desktop.'}</p>}
  </div>
 </Card>

 <Card title="Testar voz">
  <p className="notice"><ShieldAlert size={16}/><span>O teste muda o microfone real enquanto durar e pode ser ouvido na live. O BotLive devolve a voz e o modificador ao terminar, e não mexe no OBS nem no microfone do Windows.</span></p>
  <div className="row end">
   <Field label="Duração do teste" hint={`Entre ${s.minSecs} e ${s.maxSecs} segundos · padrão ${s.defaultSecs}s`}>
    <select value={duration} disabled={s.test.active} onChange={e=>setSecs(Number(e.target.value))}>{durations.map(n=><option key={n} value={n}>{n} segundos</option>)}</select>
   </Field>
  </div>
  {s.test.active&&<div className="list-row"><div><strong>Testando {s.test.voiceName||s.test.voiceId}</strong><small>{s.test.interrupted?'A conexão caiu: ao terminar a restauração fica pendente até você pedir':'Restam '+left+'s. O relógio roda no backend, não na tela.'}</small></div><span className="test-timer" aria-live="polite">{left}s</span></div>}
  {s.outcome.kind&&<p className={'outcome '+tone(s.outcome.kind)}>{s.outcome.detail}</p>}
  <div className="row">
   {s.test.active
    ?<button className="danger" disabled={!desktop||busy} onClick={()=>void stopTest()}><Square size={15}/>Encerrar teste e restaurar</button>
    :<button className="primary" disabled={!desktop||busy||!selected||pending} onClick={()=>void startTest()}><Play size={15}/>Testar voz</button>}
   {s.outcome.kind==='unconfirmed'&&<button disabled={!desktop||busy} onClick={()=>void recover()}><Undo2 size={15}/>Restaurar agora</button>}
  </div>
  {!selected&&!s.test.active&&<p className="help">Escolha uma voz na lista acima para testar.</p>}
  {pending&&!s.test.active&&s.outcome.kind==='restoring'&&<p className="help">Restaurando o estado anterior…</p>}
  <p className="help">Cada teste dura o tempo escolhido, mostra a contagem e pode ser encerrado antes. Uma troca manual de voz durante o teste nunca é desfeita em silêncio.</p>
 </Card>
 </div>;
}
