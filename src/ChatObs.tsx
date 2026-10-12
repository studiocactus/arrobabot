import {useCallback,useEffect,useRef,useState} from 'react';
import {listen} from '@tauri-apps/api/event';
import {Check,Copy,Play,Trash2,Zap} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Field,Toggle} from './components';
import {fontGroups} from './overlayDesign';
import {
 CHAT_SAMPLES,clearChat,chatSample,chatVisible,deleteChat,deleteUserChat,layoutChat,normalizeChat,pushChat,
 type ChatConfig,type ChatMsg,type ChatSample,
} from './chatOverlay';
import './overlay-studio.css';

type OverlayInfo={token:string;port:number;url:string};
type GetResult={config:unknown;overlay:OverlayInfo};

const DIRECTIONS:[ChatConfig['direction'],string][]=[['up','Novas embaixo'],['down','Novas no topo']];
const ALIGNS:[ChatConfig['align'],string][]=[['left','Esquerda'],['center','Centro'],['right','Direita']];
const ENTERS:[ChatConfig['enter'],string][]=[['fade','Fade'],['slide','Deslize'],['pixel','Pisca'],['type','Datilografia']];
const HLMODES:[ChatConfig['hlMode'],string][]=[['none','Nada'],['bar','Barra'],['bg','Fundo'],['both','Os dois']];
const BADGE_LABELS:[keyof ChatConfig['badges'],string][]=[
 ['broadcaster','Transmissor'],['moderator','Moderador'],['vip','VIP'],['subscriber','Inscrito'],
 ['founder','Fundador'],['prime','Prime'],['bits','Bits'],['gift','Presente'],['other','Outros'],
];
const SAMPLE_LABELS:Record<ChatSample,string>={
 common:'Comum',mod:'Moderador',vip:'VIP',sub:'Sub',first:'Primeira vez',
 emotes:'Em emote',emotesOnly:'Só emotes',bot:'Do bot',
};

/**
 * Estúdio de chat: configura a camada de chat do overlay do OBS com prévia em
 * tamanho real e mensagens de teste que entram pelo mesmo motor das reais.
 * Com o app aberto, o chat ao vivo também aparece na prévia.
 */
export default function ChatObs({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [cfg,setCfg]=useState<ChatConfig>(()=>normalizeChat(null));
 const [info,setInfo]=useState<OverlayInfo|null>(null);
 const [saveState,setSaveState]=useState<'idle'|'saving'|'saved'|'error'>('idle');
 const [saveError,setSaveError]=useState('');
 const [copied,setCopied]=useState(false);
 const [live,setLive]=useState(false);
 const hydrated=useRef(false);
 const lastSaved=useRef('');
 const preview=useRef<HTMLDivElement>(null);
 const cfgRef=useRef(cfg);
 cfgRef.current=cfg;

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{
   const g=await api<GetResult>('chat.get',{profileId});
   const normalized=normalizeChat(g.config);
   setCfg(normalized);
   lastSaved.current=JSON.stringify(normalized);
   setInfo(g.overlay&&g.overlay.token?g.overlay:null);
  }catch{}
  hydrated.current=true;
 },[profileId,desktop]);
 useEffect(()=>{if(desktop)void load()},[load]);

 // Autosave curto: o backend valida e retransmite a config ao OBS na hora.
 useEffect(()=>{
  if(!desktop||!hydrated.current)return;
  const key=JSON.stringify(cfg);
  if(key===lastSaved.current)return;
  setSaveState('saving');
  let cancelled=false;
  const timer=setTimeout(async()=>{
   try{
    await api('module.config',{profileId,key:'chatOverlay',value:cfg});
    lastSaved.current=key;
    if(!cancelled)setSaveState('saved');
   }catch(e){if(!cancelled){setSaveState('error');setSaveError(errorText(e))}}
  },700);
  return()=>{cancelled=true;clearTimeout(timer)};
 },[cfg,profileId,desktop]);

 // Com o Estúdio aberto, o chat real também corre na prévia (mesmo caminho
 // do OBS) — é a forma de ver o visual sem ir ao live.
 useEffect(()=>{
  if(!desktop)return;
  let disposed=false;
  const unsubs:Promise<(()=>void)|undefined>[]=[
   listen<ChatMsg>('chat',ev=>{
    const box=preview.current;if(!box)return;
    if(chatVisible(ev.payload,cfgRef.current)){pushChat(box,ev.payload,cfgRef.current,20);setLive(true)}
   }),
   listen<{id:string}>('chat.delete',ev=>{if(preview.current)deleteChat(preview.current,ev.payload.id)}),
   listen<{userId:string}>('chat.deleteUser',ev=>{if(preview.current)deleteUserChat(preview.current,ev.payload.userId)}),
   listen('chat.clear',()=>{if(preview.current)clearChat(preview.current)}),
  ];
  return()=>{disposed=true;void Promise.all(unsubs).then(fns=>{if(!disposed)fns.forEach(f=>f&&f())})};
 },[desktop]);

 // Mudou a config: a coluna inteira se recoloca e reaparece com o novo visual.
 useEffect(()=>{if(preview.current)layoutChat(preview.current,cfg,20)},[cfg]);

 const set=(patch:Partial<ChatConfig>)=>setCfg(c=>({...c,...patch}));

 const test=(kind:ChatSample)=>{
  const box=preview.current;if(!box)return;
  const m=chatSample(kind);
  if(chatVisible(m,cfg))pushChat(box,m,cfg,20);
 };
 const sequence=async()=>{
  for(const k of CHAT_SAMPLES){test(k);await new Promise(r=>setTimeout(r,700))}
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
  <Card title="Chat no overlay" action={<span className={'status-pill '+(cfg.enabled?'on':'off')}><span className="status-dot" aria-hidden="true"></span>{cfg.enabled?'Ligado':'Desligado'}</span>}>
   <p className="help">As mensagens do chat viram uma coluna na sua fonte de navegador: badges de moderador/VIP/sub, emotes do canal, destaque por papel e sincronização com a moderação (mensagem apagada some do overlay). Usa o mesmo endereço do overlay da meta e dos alertas.</p>
   <div className="list-row">
    <div><strong>Chat no overlay</strong><small>{cfg.enabled?(live?'Mensagens reais já estão correndo na prévia':'Mensagens reais aparecem no OBS'):'Nada aparece até você ligar; os botões de teste sempre funcionam'}</small></div>
    <Toggle label="Ligar chat no overlay" checked={cfg.enabled} onChange={v=>set({enabled:v})}/>
   </div>
   {saveError&&<p role="alert" className="inline-error">{saveError}</p>}
  </Card>

  <Card title="Estúdio de chat" action={<span className={'lp-save lp-save--'+saveState}>{saveState==='saving'?'Salvando…':saveState==='saved'?'Salvo':saveState==='error'?'Não salvou':'Alterações salvam sozinhas'}</span>}>
   <p className="help">Toque nos botões para empilhar mensagens de exemplo com o visual atual. Os testes seguem os mesmos filtros de cima: escondendo bots ou comandos, os exemplos correspondentes também somem — igual ao OBS.</p>

   <div className="lp-studio">
    <div className="lp-preview">
     <div className="lp-stagebar ch-tests" role="group" aria-label="Mensagens de teste">
      {CHAT_SAMPLES.map(k=><button key={k} type="button" className="lp-seg" onClick={()=>test(k)}><Play size={12}/> {SAMPLE_LABELS[k]}</button>)}
      <button type="button" className="lp-seg" onClick={()=>void sequence()}><Zap size={13}/> Sequência</button>
      <button type="button" className="lp-seg" onClick={()=>preview.current&&clearChat(preview.current)}><Trash2 size={13}/> Limpar</button>
     </div>

     <div className="ch-stage">
      <span className="ch-stage-hint">COLUNA EM TAMANHO REAL</span>
      <div ref={preview} className="bl-chat-col" aria-hidden="true"></div>
     </div>

     <p className="help">A coluna no mesmo tamanho que o OBS desenha; o lado segue o alinhamento escolhido.{cfg.enabled&&' Com o chat ligado, as mensagens da live também entram aqui.'}</p>
    </div>

    <div className="lp-controls">
     <div className="lp-group">
      <h4>Coluna</h4>
      <Seg label="Fluxo das mensagens" value={cfg.direction} options={DIRECTIONS} onChange={v=>set({direction:v})}/>
      <RangeRow label="Máximo de mensagens" min={10} max={300} step={5} value={cfg.max} fmt={v=>String(v)} onChange={v=>set({max:v})}/>
      <RangeRow label="Some depois de" min={0} max={120} step={5} value={cfg.hideAfter} fmt={v=>v===0?'nunca':v+' s'} onChange={v=>set({hideAfter:v})}/>
      <Seg label="Entrada das mensagens" value={cfg.enter} options={ENTERS} onChange={v=>set({enter:v})}/>
      <RangeRow label="Largura da coluna" min={240} max={960} step={20} value={cfg.width} fmt={v=>v+'px'} onChange={v=>set({width:v})}/>
      <Seg label="Lado na tela" value={cfg.align} options={ALIGNS} onChange={v=>set({align:v})}/>
      <div className="lp-ctrl">
       <Toggle label="Listras alternadas no fundo" checked={cfg.stripes} onChange={v=>set({stripes:v})}/>
      </div>
     </div>

     <div className="lp-group">
      <h4>Mensagem</h4>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Fonte</span>
       <select value={cfg.font} onChange={e=>set({font:e.target.value})}>
        {fontGroups.map(g=><optgroup key={g.label} label={g.label}>{g.fonts.map(f=><option key={f} value={f}>{f}</option>)}</optgroup>)}
       </select>
       <small className="lp-note">Google Fonts exigem internet no computador do OBS; fontes do sistema funcionam offline.</small>
      </div>
      <RangeRow label="Tamanho do texto" min={10} max={28} step={1} value={cfg.fontSize} fmt={v=>v+'px'} onChange={v=>set({fontSize:v})}/>
      <RangeRow label="Tamanho do nome" min={10} max={36} step={1} value={cfg.nameSize} fmt={v=>v+'px'} onChange={v=>set({nameSize:v})}/>
      <ColorRow label="Cor do texto" value={cfg.color} onChange={v=>set({color:v})}/>
      <div className="lp-ctrl">
       <Toggle label="Nome em caixa alta" checked={cfg.upper} onChange={v=>set({upper:v})}/>
      </div>
      <div className="lp-ctrl">
       <Toggle label="Cor fixa no nome" checked={!!cfg.nameFixed} onChange={v=>set({nameFixed:v?'#ffffff':''})}/>
       {cfg.nameFixed&&<ColorRow label="Cor do nome" value={cfg.nameFixed} onChange={v=>set({nameFixed:v})}/>}
       <small className="lp-note">Desligado, cada espectador ganha uma cor estável própria (a Twitch não expõe a cor real pelo caminho que o bot usa).</small>
      </div>
     </div>

     <div className="lp-group">
      <h4>Badges</h4>
      <div className="lp-toggles">
       {BADGE_LABELS.map(([k,l])=><Toggle key={k} label={l} checked={cfg.badges[k]} onChange={v=>setCfg(c=>({...c,badges:{...c.badges,[k]:v}}))}/>)}
      </div>
     </div>

     <div className="lp-group">
      <h4>Destaques por papel</h4>
      <Seg label="Como destacar" value={cfg.hlMode} options={HLMODES} onChange={v=>set({hlMode:v})}/>
      <div className="lp-colors">
       <ColorRow label="Transmissor" value={cfg.hl.broadcaster} onChange={v=>set({hl:{...cfg.hl,broadcaster:v}})}/>
       <ColorRow label="Moderador" value={cfg.hl.moderator} onChange={v=>set({hl:{...cfg.hl,moderator:v}})}/>
       <ColorRow label="VIP" value={cfg.hl.vip} onChange={v=>set({hl:{...cfg.hl,vip:v}})}/>
       <ColorRow label="Inscrito" value={cfg.hl.subscriber} onChange={v=>set({hl:{...cfg.hl,subscriber:v}})}/>
      </div>
      <div className="lp-ctrl">
       <Toggle label="Destacar a primeira mensagem" checked={cfg.firstOn} onChange={v=>set({firstOn:v})}/>
       {cfg.firstOn&&<ColorRow label="Cor da primeira mensagem" value={cfg.firstColor} onChange={v=>set({firstColor:v})}/>}
       <small className="lp-note">“Primeira vez” é desde que o BotLive abriu, não a primeira da vida no canal.</small>
      </div>
     </div>

     <div className="lp-group">
      <h4>Filtros</h4>
      <div className="lp-ctrl">
       <Toggle label="Mostrar as mensagens do bot" checked={cfg.showBot} onChange={v=>set({showBot:v})}/>
       <Field label="Nome do bot no chat"><input value={cfg.botName} maxLength={64} onChange={e=>set({botName:e.target.value})}/></Field>
      </div>
      <div className="lp-ctrl">
       <Toggle label="Esconder outros bots" checked={cfg.hideBots} onChange={v=>set({hideBots:v})}/>
       <Field label="Lista de bots" hint="Nomes separados por vírgula; nomes terminados em “bot” já são escondidos."><input value={cfg.bots} maxLength={200} onChange={e=>set({bots:e.target.value})}/></Field>
      </div>
      <div className="lp-ctrl">
       <Toggle label="Esconder comandos (!)" checked={cfg.hideCommands} onChange={v=>set({hideCommands:v})}/>
      </div>
      <Field label="Palavras bloqueadas" hint="Separadas por vírgula; a mensagem que contiver alguma delas não aparece."><input value={cfg.blocked} maxLength={200} onChange={e=>set({blocked:e.target.value})}/></Field>
     </div>
    </div>
   </div>
  </Card>

  <Card title="No OBS" action={info?<span className="status-pill on"><span className="status-dot" aria-hidden="true"></span>Pronto</span>:undefined}>
   {info?<>
    <p className="help">É o <strong>mesmo endereço do overlay da meta e dos alertas</strong>: a camada de chat entra na fonte de navegador que você já tem no OBS. Coloque a coluna onde o chat não cubra a cena — o alinhamento acima define o lado.</p>
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

/** Grupo de botões lado a lado. */
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
