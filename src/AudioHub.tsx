import {useCallback,useEffect,useRef,useState} from 'react';
import {Play} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Field} from './components';
import AudioOutput from './AudioOutput';
import {beepDataUri,normalizeAudio,type AudioConfig} from './audio';
import {playViewerSound} from './viewerSound';

type Dev={name:string;default:boolean};
type Devices={inputs:Dev[];outputs:Dev[]};
type VoiceStatus={phase:string;message:string;input:string;output:string;effect?:string};
type Selection={input?:string;output?:string};
type GetResult={config:unknown;voiceFx:{selection:Selection;devices:Devices};status:VoiceStatus};

const PHASE:Record<string,string>={off:'Voz parada',starting:'Iniciando',running:'Voz ao vivo ativa',failed:'Falha'};

/**
 * Áudio: uma saída só para todos os sons do bot. Escolha o aparelho da linha
 * (ex.: CABLE Input do VB-Cable) e o OBS captura o bot inteiro como uma única
 * fonte de áudio. A voz da live tem caminho próprio (microfone → processamento
 * → linha) e o TTS continua preso à saída padrão do Windows.
 */
export default function AudioHub({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [cfg,setCfg]=useState<AudioConfig>(()=>normalizeAudio(null));
 const [selection,setSelection]=useState<Selection>({});
 const [devices,setDevices]=useState<Devices>({inputs:[],outputs:[]});
 const [status,setStatus]=useState<VoiceStatus>({phase:'off',message:'Prévia no navegador: a voz ao vivo só existe no aplicativo desktop.',input:'',output:''});
 const [saveState,setSaveState]=useState<'idle'|'saving'|'saved'|'error'>('idle');
 const [saveError,setSaveError]=useState('');
 const [testing,setTesting]=useState(false);
 const hydrated=useRef(false);
 const lastSaved=useRef('');

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{
   const g=await api<GetResult>('audio.get',{profileId});
   const normalized=normalizeAudio(g.config);
   setCfg(normalized);
   lastSaved.current=JSON.stringify(normalized);
   setSelection(g.voiceFx?.selection||{});
   setDevices(g.voiceFx?.devices||{inputs:[],outputs:[]});
   if(g.status)setStatus(g.status);
  }catch{}
  hydrated.current=true;
 },[profileId,desktop]);
 useEffect(()=>{if(desktop)void load()},[load]);

 // Autosave: o backend valida e guarda na hora.
 useEffect(()=>{
  if(!desktop||!hydrated.current)return;
  const key=JSON.stringify(cfg);
  if(key===lastSaved.current)return;
  setSaveState('saving');
  let cancelled=false;
  const timer=setTimeout(async()=>{
   try{
    await api('module.config',{profileId,key:'audio',value:cfg});
    lastSaved.current=key;
    if(!cancelled)setSaveState('saved');
   }catch(e){if(!cancelled){setSaveState('error');setSaveError(errorText(e))}}
  },700);
  return()=>{cancelled=true;clearTimeout(timer)};
 },[cfg,profileId,desktop]);

 const set=(patch:Partial<AudioConfig>)=>setCfg(c=>({...c,...patch}));

 /** Bipe pela saída escolhida: é o mesmo caminho de qualquer som do bot. */
 const testLine=()=>{
  setTesting(true);
  playViewerSound({profileId,src:beepDataUri(),volume:0.8,deviceId:cfg.device})
   .then(()=>notify('Bipe tocado na saída dos sons.'))
   .catch(e=>notify(errorText(e)))
   .finally(()=>setTesting(false));
 };

 const saveVoice=async(side:'input'|'output',value:string)=>{
  const next={...selection,[side]:value};
  setSelection(next);
  try{await api('voiceFx.saveSelection',{profileId,input:next.input||'',output:next.output||''})}
  catch(e){notify(errorText(e))}
 };
 const running=status.phase==='running';

 return <div>
  <Card title="Saída dos sons do bot" action={<span className={'au-save au-save--'+saveState}>{saveState==='saving'?'Salvando…':saveState==='saved'?'Salvo':saveState==='error'?'Não salvou':'Alterações salvam sozinhas'}</span>}>
   <p className="help">Todos os sons saem por aqui — <strong>espectador, comandos, timers, automações, resgates e alertas</strong>. No OBS, adicione <strong>uma única Fonte de áudio</strong> (por exemplo, “CABLE Output”) e você controla o bot inteiro num fader só. Escolha abaixo a entrada “CABLE Input” do <strong>VB-Cable</strong>.</p>
   <AudioOutput value={cfg.device} onChange={v=>set({device:v})}/>
   <div className="row">
    <button className="primary" disabled={!desktop||testing} onClick={()=>testLine()}><Play size={15}/>{testing?'Tocando…':'Testar saída'}</button>
   </div>
   {!desktop&&<p className="help">Prévia no navegador: listar aparelhos e tocar exigem o aplicativo desktop.</p>}
   {saveError&&<p role="alert" className="inline-error">{saveError}</p>}
   <p className="help">Dica: escolha o mesmo aparelho aqui e na saída da Voz da live logo abaixo, e tudo do bot cai no mesmo cabo.</p>
  </Card>

  <Card title="Voz da live — linha do microfone" action={<span className={'status-pill '+(running?'on':'off')}><span className="status-dot" aria-hidden="true"></span>{PHASE[status.phase]||'Voz parada'}</span>}>
   <p className="help">A voz com efeitos sai por caminho próprio: microfone → processamento → linha, sempre separada dos sons e do TTS. Aqui ficam os aparelhos que a tela <strong>Voz da live</strong> usa; efeitos, corte de ruído e o botão de iniciar continuam lá.</p>
   <div className="list-row"><div><strong>Estado</strong><small>{status.message}</small></div></div>
   <Field label="Microfone de entrada" hint="Seu microfone de verdade. Não escolha o CABLE aqui — ele é só a saída.">
    <select value={selection.input||''} disabled={!desktop} onChange={e=>void saveVoice('input',e.target.value)}>
     <option value="">Escolha o microfone…</option>
     {selection.input&&!devices.inputs.some(d=>d.name===selection.input)&&<option value={selection.input}>{selection.input} — confira se está conectado</option>}
     {devices.inputs.map(d=><option key={d.name} value={d.name}>{d.name}{d.default?' (padrão)':''}</option>)}
    </select>
   </Field>
   <Field label="Saída da linha (VB-Cable)" hint="Escolha a entrada “CABLE Input”: é por aí que o OBS ouve a voz alterada.">
    <select value={selection.output||''} disabled={!desktop} onChange={e=>void saveVoice('output',e.target.value)}>
     <option value="">Escolha a saída…</option>
     {selection.output&&!devices.outputs.some(d=>d.name===selection.output)&&<option value={selection.output}>{selection.output} — confira se está conectado</option>}
     {devices.outputs.map(d=><option key={d.name} value={d.name}>{d.name}{d.default?' (padrão)':''}</option>)}
    </select>
   </Field>
   <p className="help">A voz só passa pelo caminho quando você clica em Iniciar na tela Voz da live — nada muda até lá.</p>
  </Card>

  <Card title="Texto para voz (TTS)">
   <p className="help">O leitor de voz dos fluxos sai pela <strong>saída padrão do Windows</strong>: o navegador não permite ao BotLive escolher outro aparelho para essa voz. Para colocá-la na linha, troque a saída padrão do Windows ou use o Mixer de volume para rotear o áudio do BotLive.</p>
   <div className="list-row"><div><strong>Saída do TTS</strong><small>Saída padrão do Windows — não há como o app escolher outro aparelho</small></div><span className="au-fixed">Fixa</span></div>
   <div className="list-row"><div><strong>Voz e velocidade</strong><small>Configuradas em Comunidade → Texto para voz</small></div></div>
  </Card>
 </div>;
}
