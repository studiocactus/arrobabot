import {useCallback,useEffect,useRef,useState} from 'react';
import {Play} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Field,Toggle} from './components';
import AudioOutput from './AudioOutput';
import {beepDataUri,normalizeAudio,type AudioConfig} from './audio';
import {playViewerSound} from './viewerSound';

type Dev={name:string;default:boolean};
type Devices={inputs:Dev[];outputs:Dev[]};
type VoiceStatus={phase:string;message:string;input:string;output:string;effect?:string};
type Selection={input?:string;output?:string};
type GetResult={config:unknown;voiceFx:{selection:Selection;devices:Devices};status:VoiceStatus};

type OriginRow={key:string;title:string;sub:string;line:keyof AudioConfig;browser:keyof AudioConfig};
/** Cada origem de som com seus próprios interruptores de linha e navegador. */
const ORIGINS:OriginRow[]=[
 {key:'people',title:'Sons de espectador',sub:'Quem fala no chat dispara o som da pessoa (Respostas e sons).',line:'peopleLine',browser:'peopleBrowser'},
 {key:'flow',title:'Sons de fluxo',sub:'O áudio ligado a comandos, timers e automações (Automações).',line:'flowLine',browser:'flowBrowser'},
 {key:'alert',title:'Alertas da live',sub:'O som de seguidor, sub, bits e raid configurado na tela Alertas.',line:'alertLine',browser:'alertBrowser'},
];

const PHASE:Record<string,string>={off:'Voz parada',starting:'Iniciando',running:'Voz ao vivo ativa',failed:'Falha'};

/**
 * Central de Áudio: um lugar só para escolher, por origem de som, se ele sai
 * na linha de áudio (aplicativo → aparelho de saída) e/ou no navegador
 * (fonte do overlay do OBS), além do aparelho da linha e dos aparelhos da
 * Voz da live. O TTS é mostrado com sua limitação fixa.
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

 // Autosave: o backend valida e retransmite a config ao OBS na hora.
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
 const setRoute=(key:keyof AudioConfig,v:boolean)=>set({[key]:v} as Partial<AudioConfig>);

 /** Bipe pela linha escolhida: é o mesmo caminho de qualquer som marcado "Linha". */
 const testLine=()=>{
  setTesting(true);
  playViewerSound({profileId,src:beepDataUri(),volume:0.8,deviceId:cfg.device})
   .then(()=>notify('Bipe tocado na linha de áudio.'))
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
  <Card title="Para onde vai cada som" action={<span className={'au-save au-save--'+saveState}>{saveState==='saving'?'Salvando…':saveState==='saved'?'Salvo':saveState==='error'?'Não salvou':'Alterações salvam sozinhas'}</span>}>
   <p className="help">Todos os sons do bot em um lugar só. Cada origem pode sair pela <strong>linha de áudio</strong> — o aplicativo toca no aparelho de saída escolhido aqui embaixo, aponte para um cabo virtual (VB-Cable) e capture no OBS — e/ou pelo <strong>navegador</strong>, quando a fonte do overlay toca o som dentro do OBS. Os dois podem estar ligados ao mesmo tempo.</p>
   {ORIGINS.map(o=>{
    const line=cfg[o.line] as boolean;const browser=cfg[o.browser] as boolean;
    return <div className="list-row" key={o.key}>
     <div><strong>{o.title}</strong><small>{!line&&!browser?'Os dois caminhos desligados: nada toca até você ligar um deles.':o.sub}</small></div>
     <div className="au-routes">
      <span className="au-route"><span>Linha</span><Toggle label={`${o.title} na linha de áudio`} checked={line} onChange={v=>setRoute(o.line,v)}/></span>
      <span className="au-route"><span>Navegador</span><Toggle label={`${o.title} no navegador`} checked={browser} onChange={v=>setRoute(o.browser,v)}/></span>
     </div>
    </div>;
   })}
   {saveError&&<p role="alert" className="inline-error">{saveError}</p>}
  </Card>

  <Card title="Linha de áudio (saída do aplicativo)">
   <p className="help">É por aqui que saem os sons marcados <strong>Linha</strong>: o BotLive toca no aparelho escolhido, sem passar pela saída padrão do Windows. Escolha a entrada “CABLE Input” do <strong>VB-Cable</strong> para o som cair direto no OBS.</p>
   <AudioOutput value={cfg.device} onChange={v=>set({device:v})}/>
   <div className="row">
    <button className="primary" disabled={!desktop||testing} onClick={()=>testLine()}><Play size={15}/>{testing?'Tocando…':'Testar linha'}</button>
   </div>
   {!desktop&&<p className="help">Prévia no navegador: listar aparelhos e tocar exigem o aplicativo desktop.</p>}
   <p className="help">Para alinhar tudo, escolha o mesmo aparelho (por exemplo, CABLE Input) aqui e na saída da Voz da live logo abaixo.</p>
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
   <div className="list-row"><div><strong>Saída do TTS</strong><small>Saída padrão do Windows — não há interruptor de linha nem de navegador</small></div><span className="au-fixed">Fixa</span></div>
   <div className="list-row"><div><strong>Voz e velocidade</strong><small>Configuradas em Comunidade → Texto para voz</small></div></div>
  </Card>
 </div>;
}
