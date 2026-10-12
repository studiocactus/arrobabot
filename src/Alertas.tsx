import {useCallback,useEffect,useRef,useState} from 'react';
import {Check,Copy,Play} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Field,Toggle} from './components';
import {fontGroups} from './overlayDesign';
import {
 ALERT_KINDS,ALERT_LABELS,alertSoundUrl,normalizeAlerts,queueAlert,renderStatic,sampleVars,
 type AlertKind,type AlertsConfig,
} from './alerts';
import './overlay-studio.css';

type Sound={id:string;name:string};
type OverlayInfo={token:string;port:number;url:string};
type GetResult={config:unknown;sounds:Sound[];overlay:OverlayInfo};

type PosV=AlertsConfig['posV'];
type PosH=AlertsConfig['posH'];
const POSV:[PosV,string][]=[['top','Topo'],['center','Meio'],['bottom','Base']];
const POSH:[PosH,string][]=[['left','Esquerda'],['center','Centro'],['right','Direita']];

/**
 * Estúdio de Alertas: configura a camada de alertas do overlay do OBS com
 * prévia animada e botões de teste por evento. O teste entra pelo mesmo
 * caminho do evento real — o OBS toca junto com a prévia do app.
 */
export default function Alertas({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [cfg,setCfg]=useState<AlertsConfig>(()=>normalizeAlerts(null));
 const [sounds,setSounds]=useState<Sound[]>([]);
 const [info,setInfo]=useState<OverlayInfo|null>(null);
 const [sel,setSel]=useState<AlertKind>('follow');
 const [saveState,setSaveState]=useState<'idle'|'saving'|'saved'|'error'>('idle');
 const [saveError,setSaveError]=useState('');
 const [copied,setCopied]=useState(false);
 const [animating,setAnimating]=useState(false);
 const [testing,setTesting]=useState(false);
 const hydrated=useRef(false);
 const lastSaved=useRef('');
 const frame=useRef<HTMLDivElement>(null);
 const stage=useRef<HTMLDivElement>(null);
 const [scale,setScale]=useState(0.3);

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{
   const g=await api<GetResult>('alerts.get',{profileId});
   const normalized=normalizeAlerts(g.config);
   setCfg(normalized);
   lastSaved.current=JSON.stringify(normalized);
   setSounds(g.sounds||[]);
   setInfo(g.overlay&&g.overlay.token?g.overlay:null);
  }catch{}
  hydrated.current=true;
 },[profileId,desktop]);
 useEffect(()=>{if(desktop)void load()},[load]);

 useEffect(()=>{
  const el=frame.current;if(!el)return;
  const fit=()=>setScale(Math.max(0.05,el.clientWidth/1920));
  fit();
  const ro=new ResizeObserver(fit);ro.observe(el);
  return()=>ro.disconnect();
 },[]);

 // Prévia parada redesenhada quando nenhum alerta está animando.
 useEffect(()=>{
  if(animating||!stage.current)return;
  renderStatic(stage.current,cfg,sel,sampleVars(sel));
 },[cfg,sel,animating]);

 // Autosave curto: o backend valida e retransmite a config ao OBS na hora.
 useEffect(()=>{
  if(!desktop||!hydrated.current)return;
  const key=JSON.stringify(cfg);
  if(key===lastSaved.current)return;
  setSaveState('saving');
  let cancelled=false;
  const timer=setTimeout(async()=>{
   try{
    await api('module.config',{profileId,key:'alerts',value:cfg});
    lastSaved.current=key;
    if(!cancelled)setSaveState('saved');
   }catch(e){if(!cancelled){setSaveState('error');setSaveError(errorText(e))}}
  },700);
  return()=>{cancelled=true;clearTimeout(timer)};
 },[cfg,profileId,desktop]);

 const set=(patch:Partial<AlertsConfig>)=>setCfg(c=>({...c,...patch}));
 const ev=cfg.events[sel];
 const setEv=(patch:Partial<AlertsConfig['events'][AlertKind]>)=>
  setCfg(c=>({...c,events:{...c.events,[sel]:{...c.events[sel],...patch}}}));

 const test=async()=>{
  if(testing)return;
  setTesting(true);setAnimating(true);
  try{
   // Mesmo caminho do evento real: o OBS toca este alerta junto com a prévia.
   const r=await api<{kind:AlertKind;vars:Record<string,unknown>}>('alerts.test',{profileId,kind:sel});
   if(stage.current){
    const sound=cfg.events[r.kind].sound;
    await queueAlert(stage.current,{
     cfg,kind:r.kind,vars:r.vars,
     audio:sound&&info?{src:alertSoundUrl(info.port,info.token,sound),volume:cfg.volume}:null,
    });
   }
  }catch(e){notify(errorText(e))}
  setTesting(false);setAnimating(false);
 };

 const copyUrl=async()=>{
  if(!info)return;
  try{
   await navigator.clipboard.writeText(info.url);
   setCopied(true);setTimeout(()=>setCopied(false),2200);
   notify('Endereço do overlay copiado.');
  }catch{notify('Não foi possível copiar. Selecione o endereço e copie manualmente.')}
 };

 return <div>
  <Card title="Alertas da live" action={<span className={'status-pill '+(cfg.enabled?'on':'off')}><span className="status-dot" aria-hidden="true"></span>{cfg.enabled?'Ligados':'Desligados'}</span>}>
   <p className="help">Follows, subs, presentes, bits e raids viram painéis animados na sua fonte de navegador. Tudo acontece no mesmo endereço do overlay da meta — ligar os alertas não exige trocar a fonte no OBS.</p>
   <div className="list-row">
    <div><strong>Alertas no overlay</strong><small>{cfg.enabled?'Eventos reais tocam no OBS':'Nada toca até você ligar; os botões de teste sempre funcionam'}</small></div>
    <Toggle label="Ligar alertas no overlay" checked={cfg.enabled} onChange={v=>set({enabled:v})}/>
   </div>
   {saveError&&<p role="alert" className="inline-error">{saveError}</p>}
  </Card>

  <Card title="Estúdio de alertas" action={<span className={'lp-save lp-save--'+saveState}>{saveState==='saving'?'Salvando…':saveState==='saved'?'Salvo':saveState==='error'?'Não salvou':'Alterações salvam sozinhas'}</span>}>
   <p className="help">Escolha o evento, edite os textos e toque em <strong>Testar</strong>: a prévia roda a mesma animação que o OBS desenha, e o teste também vai para a live (mesmo com os alertas desligados).</p>

   <div className="lp-studio">
    <div className="lp-preview">
     <div className="lp-stagebar">
      <div className="al-pills" role="group" aria-label="Eventos de alerta">
       {ALERT_KINDS.map(k=><button key={k} type="button" className={'lp-seg'+(sel===k?' selected':'')} aria-pressed={sel===k} onClick={()=>setSel(k)}>
        {ALERT_LABELS[k]}{!cfg.events[k].on&&<span className="al-pill-off" title="Desligado">•</span>}
       </button>)}
      </div>
      <button type="button" className="lp-seg" disabled={testing} onClick={()=>void test()}><Play size={13}/> Testar {ALERT_LABELS[sel].toLowerCase()}</button>
     </div>

     <div className="lp-frame" data-bg="dark" ref={frame}>
      <div className="lp-canvas" style={{transform:`scale(${scale})`}}>
       <div ref={stage}></div>
      </div>
     </div>

     <p className="help">Prévia do quadro de 1920×1080 em escala de {Math.round(scale*100)}%.{!cfg.enabled&&' Os alertas estão desligados: o OBS só toca quando você ativar acima (o teste é a exceção).'}</p>
    </div>

    <div className="lp-controls">
     <div className="lp-group">
      <h4>Este evento</h4>
      <div className="lp-ctrl">
       <Toggle label={'Exibir '+ALERT_LABELS[sel].toLowerCase()} checked={ev.on} onChange={v=>setEv({on:v})}/>
      </div>
      <Field label="Etiqueta" hint="A tarja colorida no topo do painel."><input value={ev.tag} maxLength={160} onChange={e=>setEv({tag:e.target.value})}/></Field>
      <Field label="Linha do nome"><input value={ev.name} maxLength={160} onChange={e=>setEv({name:e.target.value})}/></Field>
      <Field label="Linha grande"><input value={ev.big} maxLength={160} onChange={e=>setEv({big:e.target.value})}/></Field>
      <Field label="Linha final"><input value={ev.foot} maxLength={160} onChange={e=>setEv({foot:e.target.value})}/></Field>
      <p className="lp-note">Variáveis em qualquer texto: <code>{'{{user}}'}</code> <code>{'{{amount}}'}</code> <code>{'{{months}}'}</code> <code>{'{{tier}}'}</code> <code>{'{{sender}}'}</code> <code>{'{{message}}'}</code>. Vazias viram espaço em branco.</p>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Som</span>
       <select value={ev.sound} aria-label="Som do evento" onChange={e=>setEv({sound:e.target.value})}>
        <option value="">Sem som</option>
        {sounds.map(s=><option key={s.id} value={s.id}>{s.name}</option>)}
       </select>
       <small className="lp-note">{sounds.length?'Sons enviados na tela Respostas e sons; tocam pelo navegador do OBS.':'Nenhum som neste perfil ainda — envie um WAV, MP3 ou OGG na tela Respostas e sons.'}</small>
      </div>
     </div>

     <div className="lp-group">
      <h4>Painel</h4>
      <RangeRow label="Duração" min={3} max={20} step={1} value={cfg.duration} fmt={v=>v+' s'} onChange={v=>set({duration:v})}/>
      <RangeRow label="Velocidade das animações" min={50} max={200} step={5} value={cfg.speed} fmt={v=>v+'%'} onChange={v=>set({speed:v})}/>
      <RangeRow label="Volume do som" min={0} max={100} step={5} value={cfg.volume} fmt={v=>v+'%'} onChange={v=>set({volume:v})}/>
      <Seg label="Altura na tela" value={cfg.posV} options={POSV} onChange={v=>set({posV:v})}/>
      <Seg label="Lado na tela" value={cfg.posH} options={POSH} onChange={v=>set({posH:v})}/>
      <RangeRow label="Margem lateral" min={0} max={400} step={4} value={cfg.marginX} fmt={v=>v+'px'} onChange={v=>set({marginX:v})}/>
      <RangeRow label="Margem vertical" min={0} max={400} step={4} value={cfg.marginY} fmt={v=>v+'px'} onChange={v=>set({marginY:v})}/>
      <RangeRow label="Largura do painel" min={320} max={1760} step={20} value={cfg.panelWidth} fmt={v=>v+'px'} onChange={v=>set({panelWidth:v})}/>
      <Field label="Texto da marca" hint="Canto superior esquerdo do painel."><input value={cfg.brandText} maxLength={32} onChange={e=>set({brandText:e.target.value})}/></Field>
     </div>

     <div className="lp-group">
      <h4>Cores</h4>
      <div className="lp-colors">
       <ColorRow label="Destaque" value={cfg.accent} onChange={v=>set({accent:v})}/>
       <ColorRow label="Texto" value={cfg.text} onChange={v=>set({text:v})}/>
       <ColorRow label="Fundo do painel" value={cfg.bg} onChange={v=>set({bg:v})}/>
       <ColorRow label="Linhas" value={cfg.line} onChange={v=>set({line:v})}/>
      </div>
     </div>

     <div className="lp-group">
      <h4>Fontes</h4>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Fonte pequena (barra e etiqueta)</span>
       <select value={cfg.fontSmall} onChange={e=>set({fontSmall:e.target.value})}>
        {fontGroups.map(g=><optgroup key={g.label} label={g.label}>{g.fonts.map(f=><option key={f} value={f}>{f}</option>)}</optgroup>)}
       </select>
      </div>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Fonte grande (nomes e linhas)</span>
       <select value={cfg.fontBig} onChange={e=>set({fontBig:e.target.value})}>
        {fontGroups.map(g=><optgroup key={g.label} label={g.label}>{g.fonts.map(f=><option key={f} value={f}>{f}</option>)}</optgroup>)}
       </select>
       <small className="lp-note">Google Fonts exigem internet no computador do OBS; fontes do sistema funcionam offline.</small>
      </div>
      <RangeRow label="Tamanho da etiqueta" min={8} max={40} step={1} value={cfg.tagSize} fmt={v=>v+'px'} onChange={v=>set({tagSize:v})}/>
      <RangeRow label="Tamanho do nome" min={8} max={72} step={1} value={cfg.nameSize} fmt={v=>v+'px'} onChange={v=>set({nameSize:v})}/>
      <RangeRow label="Tamanho da linha grande" min={8} max={72} step={1} value={cfg.bigSize} fmt={v=>v+'px'} onChange={v=>set({bigSize:v})}/>
      <RangeRow label="Tamanho da linha final" min={8} max={40} step={1} value={cfg.footSize} fmt={v=>v+'px'} onChange={v=>set({footSize:v})}/>
     </div>

     <div className="lp-group">
      <h4>Efeitos</h4>
      <div className="lp-toggles">
       <Toggle label="Linhas de scanline" checked={cfg.showScanlines} onChange={v=>set({showScanlines:v})}/>
       <Toggle label="Cantos da moldura" checked={cfg.showCorners} onChange={v=>set({showCorners:v})}/>
       <Toggle label="Partículas no impacto" checked={cfg.showParticles} onChange={v=>set({showParticles:v})}/>
       <Toggle label="Glitch no impacto" checked={cfg.showGlitch} onChange={v=>set({showGlitch:v})}/>
      </div>
     </div>
    </div>
   </div>
  </Card>

  <Card title="No OBS" action={info?<span className="status-pill on"><span className="status-dot" aria-hidden="true"></span>Pronto</span>:undefined}>
   {info?<>
    <p className="help">É o <strong>mesmo endereço do overlay da meta</strong>: a camada de alertas usa a fonte de navegador que você já tem no OBS. Se ainda não adicionou, siga os passos abaixo.</p>
    <div className="lp-url">
     <code>{info.url}</code>
     <button className="primary" onClick={()=>void copyUrl()}>{copied?<Check size={15}/>:<Copy size={15}/>}{copied?'Copiado':'Copiar endereço'}</button>
    </div>
    <ol className="lp-steps">
     <li>No OBS: <strong>Fontes → + → Navegador</strong>.</li>
     <li>Cole o endereço acima (o <code>#</code> traz a chave e o perfil).</li>
     <li>Defina <strong>Largura 1920</strong> e <strong>Altura 1080</strong>.</li>
     <li>Deixe <strong>Atualizar navegador quando a cena ficar ativa</strong> desmarcado: o overlay reconecta sozinho.</li>
    </ol>
    <p className="help">As alterações do Estúdio chegam ao OBS na hora, sem recarregar a fonte.</p>
   </>:<p className="help">{desktop?'Gerando o endereço do overlay…':'O endereço do overlay é gerado pelo aplicativo desktop. Abra o BotLive no computador da live para copiar o endereço.'}</p>}
  </Card>
 </div>;
}

/** Grupo de botões lado a lado (posição na tela). */
function Seg<T extends string>({label,value,options,onChange}:{label:string;value:T;options:[T,string][];onChange:(v:T)=>void}){
 return <div className="lp-ctrl">
  <span className="lp-ctrl-label">{label}</span>
  <div className="lp-segrow" role="group" aria-label={label}>
   {options.map(([v,l])=><button key={v} type="button" className={'lp-seg'+(value===v?' selected':'')} aria-pressed={value===v} onClick={()=>onChange(v)}>{l}</button>)}
  </div>
 </div>;
}

function RangeRow({label,min,max,step,value,fmt,onChange}:{label:string;min:number;max:number;step:number;value:number;fmt:(v:number)=>string;onChange:(v:number)=>void}){
 return <div className="lp-range">
  <span className="lp-range-label">{label}<output>{fmt(value)}</output></span>
  <input type="range" min={min} max={max} step={step} value={value} aria-label={label} onChange={e=>onChange(Number(e.target.value))}/>
 </div>;
}

function ColorRow({label,value,onChange}:{label:string;value:string;onChange:(v:string)=>void}){
 const rgb=/^#[0-9a-fA-F]{6}$/.test(value)?value:'#101721';
 return <div className="lp-color">
  <span>{label}</span>
  <input type="color" value={rgb} aria-label={label} onChange={e=>onChange(e.target.value)}/>
 </div>;
}
