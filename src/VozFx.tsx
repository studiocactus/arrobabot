import {useCallback,useEffect,useRef,useState} from 'react';
import {Mic,MicOff,Play,RefreshCw,Square} from 'lucide-react';
import {listen} from '@tauri-apps/api/event';
import {api,desktop,errorText} from './api';
import {Card,Field} from './components';
import {voiceEffects} from './types';

type Status={phase:string;message:string;input:string;output:string;effect?:string;gate_db?:number};
type Dev={name:string;default:boolean};
type Devices={inputs:Dev[];outputs:Dev[]};

const PHASE:{[k:string]:{cls:string;text:string}}={
 off:{cls:'status-pill off',text:'Voz parada'},
 starting:{cls:'status-pill wait',text:'Iniciando'},
 running:{cls:'status-pill on',text:'Voz ao vivo ativa'},
 failed:{cls:'status-pill off',text:'Falha'}
};

const BLANK:Status={phase:'off',message:'Prévia no navegador: a voz ao vivo só existe no aplicativo desktop.',input:'',output:''};

export default function VozFx({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [status,setStatus]=useState<Status>(BLANK);
 const [devices,setDevices]=useState<Devices>({inputs:[],outputs:[]});
 const [input,setInput]=useState('');
 const [output,setOutput]=useState('');
 const [level,setLevel]=useState(0);
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const [gateDb,setGateDb]=useState(-45);
 const gateDirty=useRef(false);

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{setStatus(await api<Status>('voiceFx.get',{profileId}))}catch{}
  try{setDevices(await api<Devices>('voiceFx.devices',{profileId}))}catch{}
  try{const s=await api<{input?:string;output?:string;gateDb?:number}>('voiceFx.selection',{profileId});
   if(s?.input)setInput(s.input);
   if(s?.output)setOutput(s.output);
   if(typeof s?.gateDb==='number')setGateDb(s.gateDb);
  }catch{}
 },[profileId]);
 useEffect(()=>{if(desktop)void load()},[load]);

 // O corte de ruído vale na hora; salvamos com um respiro para não enviar a cada pixel do controle.
 useEffect(()=>{
  if(!desktop||!gateDirty.current)return;
  const t=setTimeout(()=>{gateDirty.current=false;void api('voiceFx.gate',{profileId,db:gateDb}).catch(()=>{})},250);
  return()=>clearTimeout(t);
 },[gateDb,profileId,desktop]);

 useEffect(()=>{
  if(!desktop)return;
  let alive=true;const off:(()=>void)[]=[];
  listen<Partial<Status>>('voiceFx',e=>{if(alive)setStatus(s=>({...s,...e.payload}))}).then(f=>{if(alive)off.push(f);else f()}).catch(()=>{});
  return()=>{alive=false;off.forEach(f=>f())};
 },[]);

 // Medidor de nível: só consulta enquanto a voz está ativa, para não poluir.
 useEffect(()=>{
  if(!desktop||status.phase!=='running')return;
  const id=setInterval(()=>{api<{level:number}>('voiceFx.level',{profileId}).then(v=>setLevel(v?.level??0)).catch(()=>{})},120);
  return()=>clearInterval(id);
 },[desktop,status.phase,profileId]);

 async function run(fn:()=>Promise<void>){
  setBusy(true);setError('');
  try{await fn()}catch(e){setError(errorText(e))}
  try{setStatus(await api<Status>('voiceFx.get',{profileId}))}catch{}
  setBusy(false);
 }

 const start=()=>run(async()=>{
  if(!input||!output){setError('Escolha o microfone e a saída do VB-Cable.');return}
  await api('voiceFx.start',{profileId,input,output});
  await api('voiceFx.saveSelection',{profileId,input,output});
  notify('Voz ao vivo iniciada. Fale e se ouvir no OBS.');
 });
 const stop=()=>run(async()=>{await api('voiceFx.stop',{profileId});notify('Voz ao vivo parada.');setLevel(0)});
 const refresh=()=>run(async()=>{setDevices(await api<Devices>('voiceFx.devices',{profileId}))});
 const setFx=(name:string)=>run(async()=>{await api('voiceFx.effect',{profileId,name})});
 const commitGate=(v:number)=>{gateDirty.current=true;setGateDb(v)};

 const phase=PHASE[status.phase]||PHASE.off;
 const running=status.phase==='running';
 const pct=Math.round(Math.min(1,Math.max(0,level))*100);

 return <div>
 <Card title="Voz da live nesta máquina">
  <p className="help">O BotLive captura o seu microfone e envia a voz para o <strong>VB-Cable</strong>, que o OBS usa como microfone. Assim trocamos sua voz ao vivo sem abrir outro programa — só um driver leve instalado uma vez. Isto não muda a voz do bot (TTS) nem mexe no OBS ou no microfone do Windows.</p>
  <div className="list-row"><div><strong>Estado</strong><small>{status.message}</small></div><span className={phase.cls}><span className="status-dot" aria-hidden="true"></span><span>{phase.text}</span></span></div>
  <div className="row">
   {!running
    ?<button className="primary" disabled={!desktop||busy||!input||!output} onClick={()=>void start()}><Play size={15}/>Iniciar voz ao vivo</button>
    :<button className="danger" disabled={!desktop||busy} onClick={()=>void stop()}><Square size={15}/>Parar</button>}
   <button disabled={!desktop||busy} onClick={()=>void refresh()}><RefreshCw size={15}/>Atualizar aparelhos</button>
  </div>
  {error&&<p role="alert" className="inline-error">{error}</p>}
  {!desktop&&<p className="help">Prévia no navegador: listar aparelhos, iniciar e medir nível exigem o aplicativo desktop.</p>}
 </Card>

 <Card title="Aparatos de áudio">
  <Field label="Microfone de entrada" hint="Seu microfone de verdade. Não escolha o CABLE aqui — ele é só a saída.">
   <select value={input} disabled={busy||running} onChange={e=>setInput(e.target.value)}>
    <option value="">Escolha o microfone…</option>
    {devices.inputs.map(d=><option key={d.name} value={d.name}>{d.name}{d.default?' (padrão)':''}</option>)}
   </select>
  </Field>
  <Field label="Saída para o OBS (VB-Cable)" hint="Escolha a entrada “CABLE Input” do VB-Cable">
   <select value={output} disabled={busy||running} onChange={e=>setOutput(e.target.value)}>
    <option value="">Escolha a saída…</option>
    {devices.outputs.map(d=><option key={d.name} value={d.name}>{d.name}{d.default?' (padrão)':''}</option>)}
   </select>
  </Field>
  <Field label="Corte de ruído de fundo" hint="Quanto menor, mais corta. Se a sua voz sumir ou cortar no meio, suba.">
   <div className="gate-row">
    <input type="range" min={-80} max={-20} step={1} value={gateDb} disabled={!desktop} aria-label="Corte de ruído em dB" onChange={e=>commitGate(Number(e.target.value))}/>
    <span className="gate-db">{gateDb} dB</span>
   </div>
  </Field>
  <p className="help">A voz passa direto do microfone ao ar: o que o microfone captura, o OBS recebe. Este corte tira o fundo constante (ventilador, teclado). Para supressão completa, some o filtro “Supressão de ruído” do OBS na fonte CABLE Output.</p>
  {(!devices.inputs.length||!devices.outputs.length)&&<p className="help">Sem aparelhos? Confira se há microfone ligado e se o VB-Cable está instalado.</p>}
  <div className="list-row">
   <div><strong>Nível do microfone</strong><small>Fale para ver a barra mexer</small></div>
   <span className="pct-badge">{pct}%</span>
  </div>
  <div className="level-track" aria-hidden="true"><div className="level-fill" style={{width:pct+'%'}}/></div>
  {input&&/CABLE|VB-Audio|VB-Cable/i.test(input)&&<p role="alert" className="inline-error">A entrada está no VB-Cable. Escolha o seu microfone real na entrada; o CABLE é só a saída — assim você fica num circuito fechado e a sua voz não passa.</p>}
  {output&&!/CABLE|VB-Audio|VB-Cable/i.test(output)&&<p role="alert" className="inline-error">A saída escolhida não parece ser do VB-Cable. O OBS só vai ouvir a voz alterada se a saída for a entrada do VB-Cable.</p>}
  {!running&&<p className="notice"><Mic size={16}/><span>Nada é alterado até você clicar em Iniciar. Ao parar, seu microfone volta ao normal.</span></p>}
  {running&&<p className="notice"><MicOff size={16}/><span>Sua voz está sendo enviada ao VB-Cable agora. No OBS, selecione “CABLE Output” como microfone para ouvir na live.</span></p>}
 </Card>
 <Card title="Voz ao vivo (efeitos)">
  <p className="help">Teste aqui a sua voz com efeito, direto, sem precisar de resgate. Vale enquanto a voz estiver iniciada acima.</p>
  <div className="fx-row">
   {Object.entries(voiceEffects).map(([k,n])=>
    <button key={k} className={(status.effect||'normal')===k?'selected':''} disabled={!desktop||busy||!running} onClick={()=>void setFx(k)}>{n}</button>)}
  </div>
  {!running&&<p className="help">Inicie a voz ao vivo para trocar a voz no ar.</p>}
  {running&&<p className="help">Voz atual no ar: <strong>{voiceEffects[status.effect||'normal']||status.effect||'normal'}</strong></p>}
 </Card>
 </div>;
}
