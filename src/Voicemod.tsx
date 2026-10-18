import {useCallback,useEffect,useState} from 'react';
import {Plug,PlugZap,RefreshCw,KeyRound,Play,Square} from 'lucide-react';
import {listen} from '@tauri-apps/api/event';
import {api,desktop,errorText} from './api';
import {Card,Field} from './components';

/**
 * Conexão com o Voicemod Control API (WebSocket local). "Conectado" só aparece
 * depois da autorização aceita pela própria Control API — nunca antes.
 * Toda a comunicação é local (portas documentadas); a chave vai para o cofre.
 */
type VmVoice={voiceID:string;name:string;enabled?:boolean};
type VmStatus={phase:string;message:string;port:number;voices:VmVoice[];currentVoice:string;currentName:string;voiceChanger:boolean|null;hearMyself:boolean|null;license:string;attempts:number};
type VmTest={active:boolean;phase:string;voiceId:string;voiceName:string;seconds:number;remainingMs:number;interrupted:boolean;manual:boolean};
type VmSnapshot={status:VmStatus;test:VmTest;outcome:{kind:string;detail:string};defaultSecs:number;minSecs:number;maxSecs:number;hasKey:boolean;keyLen:number};

const PHASE:{[k:string]:{cls:string;text:string}}={
 off:{cls:'status-pill off',text:'Desconectado'},
 searching:{cls:'status-pill wait',text:'Procurando o Voicemod…'},
 authorizing:{cls:'status-pill wait',text:'Autorizando…'},
 connected:{cls:'status-pill on',text:'Conectado'},
 failed:{cls:'status-pill off',text:'Falha'}
};

export default function Voicemod({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [snap,setSnap]=useState<VmSnapshot|null>(null);
 const [key,setKey]=useState('');
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');

 const load=useCallback(async()=>{
  if(!desktop){setSnap(null);return}
  try{setSnap(await api<VmSnapshot>('voicemod.get',{profileId}))}catch(e){setError(errorText(e))}
 },[profileId]);
 useEffect(()=>{void load()},[load]);
 useEffect(()=>{
  let alive=true;const off:((()=>void)[])=[];
  listen('voicemod',e=>{if(!alive)return;const p=e.payload as Partial<VmSnapshot>;setSnap(s=>s?{...s,...p}:s)}).then(f=>{if(alive)off.push(f);else f()}).catch(()=>{});
  return()=>{alive=false;off.forEach(f=>f())};
 },[]);
 // Aproximando da porta de autorização, o estado muda sozinho: acompanhar.
 const phase=snap?.status.phase||'off';
 useEffect(()=>{
  if(phase!=='searching'&&phase!=='authorizing')return;
  const t=window.setInterval(()=>void load(),1500);
  return()=>window.clearInterval(t);
 },[phase,load]);
 // Contagem do teste caindo.
 useEffect(()=>{
  if(!snap?.test.active)return;
  const t=window.setInterval(()=>setSnap(s=>s?{...s,test:{...s.test,remainingMs:Math.max(0,s.test.remainingMs-500)}}:s),500);
  return()=>window.clearInterval(t);
 },[snap?.test.active]);

 async function run(fn:()=>Promise<void>){
  setBusy(true);setError('');
  try{await fn()}catch(e){setError(errorText(e))}
  await load();
  setBusy(false);
 }
 const saveKey=()=>run(async()=>{
  if(!key.trim()){setError('Cole a chave da Control API antes de salvar.');return}
  await api('voicemod.key',{profileId,value:key.trim()});
  setKey('');
  notify('Chave do Voicemod salva no cofre.');
 });
 const connect=()=>run(async()=>{await api('voicemod.connect',{profileId})});
 const disconnect=()=>run(async()=>{await api('voicemod.disconnect',{profileId})});
 const refresh=()=>run(async()=>{await api('voicemod.refresh',{profileId})});

 const status=snap?.status;
 const pill=PHASE[phase]||PHASE.off;
 const connected=phase==='connected';
 const voices=(status?.voices||[]).filter(v=>v.enabled!==false);
 const test=snap?.test;
 const [testVoice,setTestVoice]=useState('');
 const [testSecs,setTestSecs]=useState<number|undefined>(undefined);
 const startTest=()=>run(async()=>{
  const id=testVoice||(voices[0]?.voiceID||'');
  if(!id){setError('Nenhuma voz disponível para testar.');return}
  await api('voicemod.testStart',{profileId,voiceId:id,seconds:testSecs??snap?.defaultSecs??15});
  notify('Testando a voz no Voicemod. O efeito vale só enquanto o contador roda.');
 });
 const stopTest=()=>run(async()=>{await api('voicemod.testStop',{profileId})});

 return <Card title="Minha voz — Voicemod">
  <p className="help">Caminho alternativo à voz interna: o BotLive pede ao Voicemod (nesta máquina) para trocar a voz, e o Voicemod alimenta o OBS pelo microfone virtual dele. O BotLive não reprocessa nem retransmite áudio. Pede-se a chave da Control API em control-api.voicemod.net — a chave fica no cofre, nunca no chat nem no Histórico.</p>
  <h4>Como usar em 4 passos</h4>
  <ol className="help">
   <li><strong>Abra e configure o Voicemod</strong> nesta máquina, com sua voz e efeitos por lá.</li>
   <li><strong>Conecte o controle</strong>: cole a chave da Control API abaixo e clique em Conectar.</li>
   <li><strong>Aponte o OBS para o microfone virtual</strong>: na cena, use “Voicemod Microphone” como microfone. O microfone padrão do Windows <strong>não muda</strong>.</li>
   <li><strong>Teste a voz</strong> aqui embaixo: o efeito vale só durante o teste e a voz de antes volta sozinha ao terminar.</li>
  </ol>
  <p className="help">Sons do bot, alertas e texto para voz <strong>não passam pelo Voicemod</strong> — eles saem pela fonte “Áudio do BotLive” do OBS (tela Áudio no OBS). A escuta de comandos continua no seu microfone físico, antes do Voicemod.</p>
  <div className="list-row"><div><strong>Estado</strong><small>{status?.message||'Abra o Voicemod nesta máquina e clique em Conectar.'}</small></div><span className={pill.cls}><span className="status-dot" aria-hidden="true"></span><span>{pill.text}</span></span></div>
  {error&&<p role="alert" className="inline-error">{error}</p>}
  {status?.phase==='failed'&&status.message&&<p role="alert" className="inline-error">{status.message}</p>}
  {status?.phase==='failed'&&<p className="help">Confira: o Voicemod está aberto e logado nesta máquina; a chave veio de control-api.voicemod.net (menu Control API); e a licença libera a Control API. Depois clique em Conectar de novo.</p>}
  {!snap?.hasKey&&<p className="help">Sem chave salva: a conexão é barrada com “Chave da Control API ausente”.</p>}
  {snap?.hasKey&&<p className="help">Chave salva ({snap.keyLen} caracteres). Nada é exibido dela aqui.</p>}
  <Field label="Chave da Control API" hint="Cole a chave recebida da Voicemod. Ela vai só para o cofre desta máquina.">
   <input type="password" value={key} onChange={e=>setKey(e.target.value)} placeholder={snap?.hasKey?'•••••••• (já salva)':'chave da Control API'}/>
  </Field>
  <div className="row">
   <button disabled={busy||!key.trim()} onClick={()=>void saveKey()}><KeyRound size={15}/>Salvar chave</button>
   {!connected
    ?<button className="primary" disabled={!desktop||busy} onClick={()=>void connect()}><Plug size={15}/>Conectar</button>
    :<button disabled={busy} onClick={()=>void disconnect()}><PlugZap size={15}/>Desconectar</button>}
   <button disabled={!desktop||busy} onClick={()=>void refresh()}><RefreshCw size={15}/>Atualizar vozes</button>
  </div>
  {connected&&<>
   <div className="list-row"><div><strong>Voz atual no Voicemod</strong><small>{status?.currentName||status?.currentVoice||'—'}</small></div></div>
   <div className="list-row"><div><strong>Modificador de voz</strong><small>{status?.voiceChanger===true?'ligado':status?.voiceChanger===false?'desligado (algumas vozes não soam)':'desconhecido'}</small></div><div className="list-row"><div><strong>Licença</strong><small>{status?.license||'—'}</small></div></div></div>
   <h4>Testar uma voz</h4>
   <Field label="Voz do Voicemod" hint="Lista real vinda do aplicativo, com as vozes liberadas pela licença.">
    <select value={testVoice} onChange={e=>setTestVoice(e.target.value)}>
     <option value="">Escolha uma voz…</option>
     {voices.map(v=><option key={v.voiceID} value={v.voiceID}>{v.name}</option>)}
    </select>
   </Field>
   <Field label={`Duração do teste (segundos)`} hint={`Padrão ${snap?.defaultSecs??15}s. Depois o BotLive devolve a voz de antes, com confirmação.`}>
    <input type="number" min={snap?.minSecs??5} max={snap?.maxSecs??600} value={String(testSecs??snap?.defaultSecs??15)} onChange={e=>setTestSecs(Math.max(snap?.minSecs??5,Math.min(snap?.maxSecs??600,Math.round(Number(e.target.value)||0))))}/>
   </Field>
   {!test?.active
    ?<button className="primary" disabled={busy||!testVoice} onClick={()=>void startTest()}><Play size={15}/>Testar voz</button>
    :<button className="danger" disabled={busy} onClick={()=>void stopTest()}><Square size={15}/>Parar teste ({Math.ceil((test?.remainingMs??0)/1000)}s)</button>}
   {test?.active&&<p className="help">Testando <strong>{test?.voiceName||test?.voiceId}</strong>. Ao terminar, a voz de antes volta com confirmação do Voicemod.</p>}
   {test?.interrupted&&<p role="alert" className="inline-error">O teste terminou sem confirmação da restauração. Toque em Conectar e depois em Recuperar na seção de resgates.</p>}
  </>}
  {!connected&&<p className="help">A lista de vozes e o teste só aparecem depois da conexão autorizada.</p>}
  {!desktop&&<p className="help">A prévia do navegador não fala com o Voicemod; use o aplicativo desktop.</p>}
 </Card>;
}
