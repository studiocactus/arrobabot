import {useCallback,useEffect,useRef,useState} from 'react';
import {Check,Copy,Play} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {AccordionItem,Card,Check as ToggleCheck,Field,ItemStack,Toggle} from './components';
import {fontGroups} from './overlayDesign';
import {
 ALERT_KINDS,ALERT_LABELS,DEFAULT_ALERTS,normalizeAlerts,queueAlert,renderStatic,sampleVars,
 type AlertKind,type AlertsConfig,
} from './alerts';
import {applyCode} from './codeMode';
import {CodeTabs} from './CodeTabs';
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
 * caminho do evento real — o OBS toca junto com a prévia do app, e o som
 * sai na saída de áudio do bot (tela Áudio), nunca na fonte de navegador.
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
 // Acordeão das configurações: só "Este evento" abre por padrão (é o que se edita).
 const [openGroups,setOpenGroups]=useState({event:true,panel:false,colors:false,fonts:false,fx:false});
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

 // Prévia parada redesenhada quando nenhum alerta está animando — depois o
 // modo código se reencaixa por cima (o redesenho substitui os filhos).
 useEffect(()=>{
  if(animating||!stage.current)return;
  renderStatic(stage.current,cfg,sel,sampleVars(sel));
  applyCode(stage.current,cfg.code,'alerts');
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
 const flip=(k:keyof typeof openGroups)=>setOpenGroups(o=>({...o,[k]:!o[k]}));

 /** Aba Data: aplica o JSON pelo mesmo normalizador do formulário. */
 const applyData=(json:string):string|null=>{
  let parsed:unknown;
  try{parsed=JSON.parse(json)}catch(e){return 'JSON inválido: '+(e instanceof Error?e.message:'confira vírgulas e aspas.')}
  if(!parsed||typeof parsed!=='object'||Array.isArray(parsed))return 'Esperado um objeto JSON com os campos desta tela.';
  setCfg(normalizeAlerts(parsed));
  return null;
 };

 const test=async()=>{
  if(testing)return;
  setTesting(true);setAnimating(true);
  try{
   // Mesmo caminho do evento real: o OBS toca este alerta junto com a prévia.
   const r=await api<{kind:AlertKind;vars:Record<string,unknown>}>('alerts.test',{profileId,kind:sel});
   if(stage.current){
    // O som sai pela linha de áudio do app (evento alert-sound); a prévia é visual.
    await queueAlert(stage.current,{cfg,kind:r.kind,vars:r.vars,audio:null});
   }
  }catch(e){notify(errorText(e))}
  setTesting(false);setAnimating(false);
 };

 const copyUrl=async()=>{
  if(!info)return;
  try{
   await navigator.clipboard.writeText(layerUrl());
   setCopied(true);setTimeout(()=>setCopied(false),2200);
   notify('Endereço da camada de alertas copiado.');
  }catch{notify('Não foi possível copiar. Selecione o endereço e copie manualmente.')}
 };
 /** Endereço da camada de alertas: mesma base do overlay com mode=alerts. */
 const layerUrl=()=>info?info.url+'&mode=alerts':'';

 return <div>
  <Card title="Alertas da live" action={<span className={'status-pill '+(cfg.enabled?'on':'off')}><span className="status-dot" aria-hidden="true"></span>{cfg.enabled?'Ligados':'Desligados'}</span>}>
   <p className="help">Follows, subs, presentes, bits e raids viram painéis animados na sua fonte de navegador. A camada de alertas tem um <strong>endereço próprio</strong> (com <code>mode=alerts</code>): crie uma Fonte de navegador só para ela e ajuste posição e tamanho sem mexer na meta nem no chat.</p>
   <div className="list-row">
    <div><strong>Alertas no overlay</strong><small>{cfg.enabled?'Eventos reais tocam no OBS':'Nada toca até você ligar; os botões de teste sempre funcionam'}</small></div>
    <Toggle label="Ligar alertas no overlay" checked={cfg.enabled} onChange={v=>set({enabled:v})}/>
   </div>
   {saveError&&<p role="alert" className="inline-error">{saveError}</p>}
  </Card>

  <Card title="Estúdio de alertas" action={<span className={'lp-save lp-save--'+saveState}>{saveState==='saving'?'Salvando…':saveState==='saved'?'Salvo':saveState==='error'?'Não salvou':'Alterações salvam sozinhas'}</span>}>
   <p className="help">Escolha o evento, edite os textos e toque em <strong>Testar</strong>: a prévia roda a mesma animação que o OBS desenha, e o teste também vai para a live (mesmo com os alertas desligados).</p>

   <div className="lp-studio lp-studio--stack">
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
       <div className="al-stage" ref={stage}></div>
      </div>
     </div>

     <p className="help">Prévia do quadro de 1920×1080 em escala de {Math.round(scale*100)}%.{!cfg.enabled&&' Os alertas estão desligados: o OBS só toca quando você ativar acima (o teste é a exceção).'}</p>
    </div>

    <div className="lp-controls">
     <CodeTabs code={cfg.code} onChange={c=>set({code:c})}
      dataJson={JSON.stringify(cfg,null,2)} defaultData={JSON.stringify(DEFAULT_ALERTS,null,2)}
      onApplyData={applyData}>
     <ItemStack>
      <AccordionItem id="al-event" open={openGroups.event} enabled={ev.on}
       enabledLabel={'Exibir '+ALERT_LABELS[sel].toLowerCase()} title="Este evento" meta={[ALERT_LABELS[sel]]}
       onToggle={()=>flip('event')} onEnabled={v=>setEv({on:v})}>
       <p className="lp-note">Desligado, o evento chega ao log mas não desenha nada no OBS.</p>
       <Field label="Etiqueta" hint="A tarja colorida no topo do painel."><input value={ev.tag} maxLength={160} onChange={e=>setEv({tag:e.target.value})}/></Field>
       <ColorRow label="Cor da tarja" hint="Cor só desta tarja; vazio usa o destaque global do painel." value={ev.color||cfg.accent}
        onChange={v=>setEv({color:v})} onReset={ev.color?()=>setEv({color:''}):undefined}/>
       <Field label="Linha do nome" hint="Quem disparou o alerta, por padrão."><input value={ev.name} maxLength={160} onChange={e=>setEv({name:e.target.value})}/></Field>
       <Field label="Linha grande" hint="A frase de destaque do painel."><input value={ev.big} maxLength={160} onChange={e=>setEv({big:e.target.value})}/></Field>
       <Field label="Linha final" hint="A rodapé do painel — mensagens, meses, valores."><input value={ev.foot} maxLength={160} onChange={e=>setEv({foot:e.target.value})}/></Field>
       <p className="lp-note">Variáveis em qualquer texto: <code>{'{{user}}'}</code> <code>{'{{amount}}'}</code> <code>{'{{months}}'}</code> <code>{'{{tier}}'}</code> <code>{'{{sender}}'}</code> <code>{'{{message}}'}</code>. Vazias viram espaço em branco.</p>
       <div className="lp-ctrl">
        <span className="lp-ctrl-label">Som</span>
        <select value={ev.sound} aria-label="Som do evento" onChange={e=>setEv({sound:e.target.value})}>
         <option value="">Sem som</option>
         {sounds.map(s=><option key={s.id} value={s.id}>{s.name}</option>)}
        </select>
        <small className="lp-note">{sounds.length?'O som toca na saída dos sons do bot (tela Áudio) junto com o painel. Envie áudios na tela Sons de Live.':'Nenhum som neste perfil ainda — envie um WAV, MP3 ou OGG na tela Sons de Live.'}</small>
       </div>
      </AccordionItem>

      <AccordionItem id="al-panel" open={openGroups.panel} title="Painel" meta={[`${cfg.duration} s`,`${cfg.volume}%`]} onToggle={()=>flip('panel')}>
       <RangeRow label="Duração" hint="Por quanto tempo o painel fica na tela." min={3} max={20} step={1} value={cfg.duration} fmt={v=>v+' s'} onChange={v=>set({duration:v})}/>
       <RangeRow label="Velocidade das animações" hint="100% = normal; menos é mais lento." min={50} max={200} step={5} value={cfg.speed} fmt={v=>v+'%'} onChange={v=>set({speed:v})}/>
       <RangeRow label="Volume do som" hint="Volume do som do alerta na saída dos sons do bot." min={0} max={100} step={5} value={cfg.volume} fmt={v=>v+'%'} onChange={v=>set({volume:v})}/>
       <Seg label="Altura na tela" hint="Onde o painel aparece na vertical." value={cfg.posV} options={POSV} onChange={v=>set({posV:v})}/>
       <Seg label="Lado na tela" hint="Onde o painel aparece na horizontal." value={cfg.posH} options={POSH} onChange={v=>set({posH:v})}/>
       <RangeRow label="Margem lateral" hint="Distância até a borda esquerda/direita." min={0} max={400} step={4} value={cfg.marginX} fmt={v=>v+'px'} onChange={v=>set({marginX:v})}/>
       <RangeRow label="Margem vertical" hint="Distância até o topo/base da cena." min={0} max={400} step={4} value={cfg.marginY} fmt={v=>v+'px'} onChange={v=>set({marginY:v})}/>
       <RangeRow label="Largura do painel" hint="Largura máxima do painel dentro da cena 1920×1080." min={320} max={1760} step={20} value={cfg.panelWidth} fmt={v=>v+'px'} onChange={v=>set({panelWidth:v})}/>
       <Field label="Texto da marca" hint="Canto superior esquerdo do painel."><input value={cfg.brandText} maxLength={32} onChange={e=>set({brandText:e.target.value})}/></Field>
      </AccordionItem>

      <AccordionItem id="al-colors" open={openGroups.colors} title="Cores" meta={[cfg.accent]} onToggle={()=>flip('colors')}>
       <div className="lp-colors">
        <ColorRow label="Destaque" hint="Cor da etiqueta e dos detalhes." value={cfg.accent} onChange={v=>set({accent:v})}/>
        <ColorRow label="Texto" hint="Todas as linhas escritas do painel." value={cfg.text} onChange={v=>set({text:v})}/>
        <ColorRow label="Fundo do painel" hint="Fundo atrás das informações." value={cfg.bg} onChange={v=>set({bg:v})}/>
        <ColorRow label="Linhas" hint="Bordas e divisórias do painel." value={cfg.line} onChange={v=>set({line:v})}/>
       </div>
      </AccordionItem>

      <AccordionItem id="al-fonts" open={openGroups.fonts} title="Fontes" meta={[cfg.fontBig]} onToggle={()=>flip('fonts')}>
       <div className="lp-ctrl">
        <span className="lp-ctrl-label">Fonte pequena (barra e etiqueta)</span>
        <select value={cfg.fontSmall} aria-label="Fonte pequena" onChange={e=>set({fontSmall:e.target.value})}>
         {fontGroups.map(g=><optgroup key={g.label} label={g.label}>{g.fonts.map(f=><option key={f} value={f}>{f}</option>)}</optgroup>)}
        </select>
       </div>
       <div className="lp-ctrl">
        <span className="lp-ctrl-label">Fonte grande (nomes e linhas)</span>
        <select value={cfg.fontBig} aria-label="Fonte grande" onChange={e=>set({fontBig:e.target.value})}>
         {fontGroups.map(g=><optgroup key={g.label} label={g.label}>{g.fonts.map(f=><option key={f} value={f}>{f}</option>)}</optgroup>)}
        </select>
        <small className="lp-note">Google Fonts exigem internet no computador do OBS; fontes do sistema funcionam offline.</small>
       </div>
       <RangeRow label="Tamanho da etiqueta" hint="Letras da tarja colorida no topo." min={8} max={40} step={1} value={cfg.tagSize} fmt={v=>v+'px'} onChange={v=>set({tagSize:v})}/>
       <RangeRow label="Tamanho do nome" hint="Nome de quem disparou o alerta." min={8} max={72} step={1} value={cfg.nameSize} fmt={v=>v+'px'} onChange={v=>set({nameSize:v})}/>
       <RangeRow label="Tamanho da linha grande" hint="Frase de destaque do painel." min={8} max={72} step={1} value={cfg.bigSize} fmt={v=>v+'px'} onChange={v=>set({bigSize:v})}/>
       <RangeRow label="Tamanho da linha final" hint="Rodapé com mensagem, meses ou valores." min={8} max={40} step={1} value={cfg.footSize} fmt={v=>v+'px'} onChange={v=>set({footSize:v})}/>
      </AccordionItem>

      <AccordionItem id="al-fx" open={openGroups.fx} title="Efeitos"
       meta={[`${[cfg.showScanlines,cfg.showCorners,cfg.showParticles,cfg.showGlitch].filter(Boolean).length} de 4`]}
       onToggle={()=>flip('fx')}>
       <div className="lp-toggles">
        <ToggleCheck label="Linhas de scanline" hint="Textura de TV antiga no painel." checked={cfg.showScanlines} onChange={v=>set({showScanlines:v})}/>
        <ToggleCheck label="Cantos da moldura" hint="Cantos decorativos nos quatro extremos." checked={cfg.showCorners} onChange={v=>set({showCorners:v})}/>
        <ToggleCheck label="Partículas no impacto" hint="Faíscas quando o painel entra." checked={cfg.showParticles} onChange={v=>set({showParticles:v})}/>
        <ToggleCheck label="Glitch no impacto" hint="Tremida curta na entrada do painel." checked={cfg.showGlitch} onChange={v=>set({showGlitch:v})}/>
       </div>
      </AccordionItem>
     </ItemStack>
     </CodeTabs>
    </div>
   </div>
  </Card>

  <Card title="No OBS" action={info?<span className="status-pill on"><span className="status-dot" aria-hidden="true"></span>Pronto</span>:undefined}>
   {info?<>
    <p className="help">Este é o <strong>endereço da camada de alertas</strong> (o parâmetro <code>mode=alerts</code> deixa só os alertas nesta fonte). Cada camada — meta, alertas e chat — fica na sua própria Fonte de navegador, com posição e tamanho independentes.</p>
    <div className="lp-url">
     <code>{layerUrl()}</code>
     <button className="primary" onClick={()=>void copyUrl()}>{copied?<Check size={15}/>:<Copy size={15}/>}{copied?'Copiado':'Copiar endereço'}</button>
    </div>
    <ol className="lp-steps">
     <li>No OBS: <strong>Fontes → + → Navegador</strong>.</li>
     <li>Cole o endereço acima (o <code>#</code> traz a chave, o perfil e a camada).</li>
     <li>Defina <strong>Largura 1920</strong> e <strong>Altura 1080</strong>.</li>
     <li>Deixe <strong>Atualizar navegador quando a cena ficar ativa</strong> desmarcado: o overlay reconecta sozinho.</li>
    </ol>
    <p className="help">As alterações do Estúdio chegam ao OBS na hora, sem recarregar a fonte.</p>
   </>:<p className="help">{desktop?'Gerando o endereço do overlay…':'O endereço do overlay é gerado pelo aplicativo desktop. Abra o BotLive no computador da live para copiar o endereço.'}</p>}
  </Card>
 </div>;
}

/** Grupo de botões lado a lado (posição na tela). */
function Seg<T extends string>({label,hint,value,options,onChange}:{label:string;hint?:string;value:T;options:[T,string][];onChange:(v:T)=>void}){
 return <div className="lp-ctrl">
  <span className="lp-ctrl-label">{label}</span>
  <div className="lp-segrow" role="group" aria-label={label}>
   {options.map(([v,l])=><button key={v} type="button" className={'lp-seg'+(value===v?' selected':'')} aria-pressed={value===v} onClick={()=>onChange(v)}>{l}</button>)}
  </div>
  {hint&&<small className="lp-note">{hint}</small>}
 </div>;
}

function RangeRow({label,hint,min,max,step,value,fmt,onChange}:{label:string;hint?:string;min:number;max:number;step:number;value:number;fmt:(v:number)=>string;onChange:(v:number)=>void}){
 return <div className="lp-range">
  <span className="lp-range-label">{label}<output>{fmt(value)}</output></span>
  <input type="range" min={min} max={max} step={step} value={value} aria-label={label} onChange={e=>onChange(Number(e.target.value))}/>
  {hint&&<small className="lp-note">{hint}</small>}
 </div>;
}

function ColorRow({label,hint,value,onChange,onReset}:{label:string;hint?:string;value:string;onChange:(v:string)=>void;onReset?:()=>void}){
 const rgb=/^#[0-9a-fA-F]{6}$/.test(value)?value:'#101721';
 return <div className="lp-color">
  <span>{label}</span>
  <input type="color" value={rgb} aria-label={label} onChange={e=>onChange(e.target.value)}/>
  {onReset&&<button type="button" className="text-button" onClick={onReset}>Usar cor global</button>}
  {hint&&<small className="lp-note">{hint}</small>}
 </div>;
}
