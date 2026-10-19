import {useCallback,useEffect,useRef,useState} from 'react';
import {listen} from '@tauri-apps/api/event';
import {Check,Copy,Play,Trash2,Zap} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Check as ToggleCheck,Field,MissingOption,Toggle} from './components';
import {fontGroups} from './overlayDesign';
import {
 CHAT_EVENT_KINDS,CHAT_EVENT_LABELS,CHAT_SAMPLES,DEFAULT_CHAT,clearChat,chatSample,chatVisible,
 deleteChat,deleteUserChat,eventBoxHtml,isChatEventKind,layoutChat,normalizeChat,pushChat,pushChatEvent,
 type ChatConfig,type ChatEventKind,type ChatMsg,type ChatSample,
} from './chatOverlay';
import {applyCode} from './codeMode';
import {CodeTabs} from './CodeTabs';
import './overlay-studio.css';

type OverlayInfo={token:string;port:number;url:string};
type GetResult={config:unknown;overlay:OverlayInfo};

const DIRECTIONS:[ChatConfig['direction'],string][]=[['up','Novas embaixo'],['down','Novas no topo']];
const ALIGNS:[ChatConfig['align'],string][]=[['left','Esquerda'],['center','Centro'],['right','Direita']];
const ENTERS:[ChatConfig['enter'],string][]=[['fade','Fade'],['slide','Deslize'],['pixel','Pisca'],['type','Datilografia']];
const HLMODES:[ChatConfig['hlMode'],string][]=[['none','Nada'],['bar','Barra'],['bg','Fundo'],['both','Os dois']];
const NAMEMODES:[ChatConfig['nameMode'],string][]=[['twitch','Twitch'],['custom','Personalizadas'],['fixed','Fixa']];
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
 const [sounds,setSounds]=useState<{id:string;name:string}[]>([]);
 const [burstChat,setBurstChat]=useState(false);
 const [evSel,setEvSel]=useState<ChatEventKind>('follow');
 const [ensaio,setEnsaio]=useState(false);
 const hydrated=useRef(false);
 const lastSaved=useRef('');
 const preview=useRef<HTMLDivElement>(null);
 const stage=useRef<HTMLDivElement>(null);
 const ensaioTimer=useRef<number|undefined>(undefined);
 const cfgRef=useRef(cfg);
 cfgRef.current=cfg;
 useEffect(()=>()=>window.clearTimeout(ensaioTimer.current),[]);

 /** Selo ENSAIO: acende quando um teste volta pelo caminho da live e some sozinho. */
 const flagEnsaio=()=>{
  setEnsaio(true);
  window.clearTimeout(ensaioTimer.current);
  ensaioTimer.current=window.setTimeout(()=>setEnsaio(false),5000);
 };

 const load=useCallback(async()=>{
  if(!desktop)return;
  try{
   const g=await api<GetResult>('chat.get',{profileId});
   const normalized=normalizeChat(g.config);
   setCfg(normalized);
   lastSaved.current=JSON.stringify(normalized);
   setInfo(g.overlay&&g.overlay.token?g.overlay:null);
   try{const b=await api<{alsoChat?:boolean}|null>('module.config.get',{profileId,key:'emoteBurst'});setBurstChat(b?.alsoChat===true)}catch{}
  }catch{}
  hydrated.current=true;
 },[profileId,desktop]);
 useEffect(()=>{if(desktop)void load()},[load]);

 /** Salva o interruptor do chat na mesma configuração da explosão (tela Alertas). */
 const saveBurstChat=async(v:boolean)=>{
  setBurstChat(v);
  try{
   const cur=await api<{alsoChat?:boolean}|null>('module.config.get',{profileId,key:'emoteBurst'});
   await api('module.config',{profileId,key:'emoteBurst',value:{...(cur||{}),alsoChat:v}});
  }catch(e){setBurstChat(!v);notify(errorText(e))}
 };

 // Sons da biblioteca para os eventos (e a saída usada na prévia).
 useEffect(()=>{
  if(!desktop)return;
  let live=true;
  api<{assets:{id:string;kind:string;name:string}[]}>('chatExtras.get',{profileId})
   .then(v=>{if(live)setSounds((v.assets||[]).filter(a=>a.kind==='sound'))})
   .catch(()=>{});
  return()=>{live=false};
 },[profileId,desktop]);

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
    if(ev.payload.rehearsal)flagEnsaio();
    const box=preview.current;if(!box)return;
    if(chatVisible(ev.payload,cfgRef.current)){pushChat(box,ev.payload,cfgRef.current,20);if(!ev.payload.rehearsal)setLive(true)}
   }),
   listen<{kind:string;text:string;color:string;rehearsal?:boolean}>('chat-event',ev=>{
    if(ev.payload.rehearsal)flagEnsaio();
    const box=preview.current;
    if(box&&isChatEventKind(ev.payload.kind))pushChatEvent(box,cfgRef.current,ev.payload.kind,ev.payload.text||'',ev.payload.color||'',20);
   }),
   listen<{id:string}>('chat-delete',ev=>{if(preview.current)deleteChat(preview.current,ev.payload.id)}),
   listen<{userId:string}>('chat-deleteUser',ev=>{if(preview.current)deleteUserChat(preview.current,ev.payload.userId)}),
   listen('chat-clear',()=>{if(preview.current)clearChat(preview.current)}),
  ];
  return()=>{disposed=true;void Promise.all(unsubs).then(fns=>{if(!disposed)fns.forEach(f=>f&&f())})};
 },[desktop]);

 // Mudou a config: a coluna inteira se recoloca e reaparece com o novo visual.
 useEffect(()=>{if(preview.current)layoutChat(preview.current,cfg,20)},[cfg]);

 // Modo código: HTML/CSS/JS entram na prévia pelo mesmo caminho do OBS.
 useEffect(()=>{if(stage.current)applyCode(stage.current,cfg.code,'chat')},[cfg.code]);

 const set=(patch:Partial<ChatConfig>)=>setCfg(c=>({...c,...patch}));
 const setEv=(k:ChatEventKind,patch:Partial<ChatConfig['ev'][ChatEventKind]>)=>
  setCfg(c=>({...c,ev:{...c.ev,[k]:{...c.ev[k],...patch}}}));

 /** Teste do evento: no aplicativo o backend preenche e transmite pelo mesmo
  *  caminho do evento da live (com o selo ENSAIO); na prévia web, desenha local. */
 const testEvent=async(k:ChatEventKind)=>{
  if(desktop){
   try{
    const out=await api<{fired?:boolean;reason?:string}>('chat.test',{profileId,mode:'event',kind:k});
    if(out?.fired===false){notify(out.reason||'Não foi possível testar o evento.');return}
   }catch(e){notify(errorText(e))}
   return;
  }
  const box=preview.current;if(!box)return;
  pushChatEvent(box,cfg,k,eventBoxHtml(cfg,k),cfg.ev[k].color,20);
  flagEnsaio();
  // Sem aplicativo não há biblioteca de sons: a prévia web fica só no visual.
  // No desktop o som sai pelo backend, pelo mesmo caminho da live.
 };

 /** Aba Data: aplica o JSON pelo mesmo normalizador do formulário. */
 const applyData=(json:string):string|null=>{
  let parsed:unknown;
  try{parsed=JSON.parse(json)}catch(e){return 'JSON inválido: '+(e instanceof Error?e.message:'confira vírgulas e aspas.')}
  if(!parsed||typeof parsed!=='object'||Array.isArray(parsed))return 'Esperado um objeto JSON com os campos desta tela.';
  setCfg(normalizeChat(parsed));
  return null;
 };

 /** Mensagem de teste: no aplicativo sai pelo `chat.test` — a amostra vira
  *  evento e o motor do chat a transmite como falaria na live (selo ENSAIO);
  *  na prévia web sem aplicativo, empilha local com o mesmo visual.
  *  Devolve `false` quando o interruptor impediu o teste (um aviso já basta). */
 const test=async(kind:ChatSample):Promise<boolean>=>{
  if(desktop){
   try{
    const out=await api<{fired?:boolean;reason?:string}>('chat.test',{profileId,mode:'message',sample:chatSample(kind)});
    if(out?.fired===false){notify(out.reason||'Não foi possível testar a mensagem.');return false}
   }catch(e){notify(errorText(e));return false}
   return true;
  }
  const box=preview.current;if(!box)return false;
  const m=chatSample(kind);
  if(chatVisible(m,cfg)){pushChat(box,m,cfg,20);flagEnsaio()}
  return true;
 };
 const sequence=async()=>{
  for(const k of CHAT_SAMPLES){
   if(!await test(k))break;
   await new Promise(r=>setTimeout(r,700));
  }
 };
 const copyUrl=async()=>{
  if(!info)return;
  try{
   await navigator.clipboard.writeText(layerUrl());
   setCopied(true);setTimeout(()=>setCopied(false),2200);
   notify('Endereço da camada de chat copiado.');
  }catch{notify('Não foi possível copiar. Selecione o endereço e copie manualmente.')}
 };
 /** Endereço da camada de chat: mesma base do overlay com mode=chat. */
 const layerUrl=()=>info?info.url+'&mode=chat':'';

 return <div>
  <Card title="Chat no overlay" action={<span className={'status-pill '+(cfg.enabled?'on':'off')}><span className="status-dot" aria-hidden="true"></span>{cfg.enabled?'Ligado':'Desligado'}</span>}>
   <p className="help">As mensagens do chat viram uma coluna na sua fonte de navegador: prefixo &gt; antes do nome, badges de moderador/VIP/sub, cores dos espectadores (a cor real da Twitch ou a sua escolha), emotes do canal, eventos de follow/sub/bits/raid na própria coluna e sincronização com a moderação (mensagem apagada some do overlay). A camada de chat tem um <strong>endereço próprio</strong> (com <code>mode=chat</code>): crie uma Fonte de navegador só para ela e coloque a coluna onde quiser, sem mexer na meta nem nos alertas.</p>
   <div className="list-row">
    <div><strong>Chat no overlay</strong><small>{cfg.enabled?(live?'Mensagens reais já estão correndo na prévia':'Mensagens reais aparecem no OBS'):'Nada aparece até você ligar — e os testes avisam que, em live, também não apareceria'}</small></div>
    <Toggle label="Ligar chat no overlay" checked={cfg.enabled} onChange={v=>set({enabled:v})}/>
   </div>
   {saveError&&<p role="alert" className="inline-error">{saveError}</p>}
  </Card>

  <Card title="Estúdio de chat" action={<span className={'lp-save lp-save--'+saveState}>{saveState==='saving'?'Salvando…':saveState==='saved'?'Salvo':saveState==='error'?'Não salvou':'Alterações salvam sozinhas'}</span>}>
   <p className="help">Toque nos botões para empilhar mensagens de exemplo com o visual atual. Com o chat ligado, os testes entram pelo mesmo caminho da live (e o palco mostra o selo Ensaio); com ele desligado, um aviso explica que em live nada apareceria. Os testes seguem os mesmos filtros de cima: escondendo bots ou comandos, os exemplos correspondentes também somem — igual ao OBS.</p>

   <div className="lp-studio">
    <div className="lp-preview">
     <div className="lp-stagebar ch-tests" role="group" aria-label="Mensagens de teste">
      {CHAT_SAMPLES.map(k=><button key={k} type="button" className="lp-seg" onClick={()=>void test(k)}><Play size={12}/> {SAMPLE_LABELS[k]}</button>)}
      <button type="button" className="lp-seg" onClick={()=>void sequence()}><Zap size={13}/> Sequência</button>
      <button type="button" className="lp-seg" onClick={()=>preview.current&&clearChat(preview.current)}><Trash2 size={13}/> Limpar</button>
     </div>

     <div className="ch-stage" ref={stage}>
      <span className="ch-stage-hint">COLUNA EM TAMANHO REAL</span>
      {ensaio&&<span className="lp-ensaio">Ensaio</span>}
      <div ref={preview} className="bl-chat-col" aria-hidden="true"></div>
     </div>

     <p className="help">A coluna no mesmo tamanho que o OBS desenha; o lado segue o alinhamento escolhido.{cfg.enabled&&' Com o chat ligado, as mensagens da live também entram aqui.'}</p>
    </div>

    <div className="lp-controls">
     <CodeTabs code={cfg.code} onChange={c=>set({code:c})}
      dataJson={JSON.stringify(cfg,null,2)} defaultData={JSON.stringify(DEFAULT_CHAT,null,2)}
      onApplyData={applyData}>
     <div className="lp-group">
      <h4>Coluna</h4>
      <Seg label="Fluxo das mensagens" hint="Novas embaixo empilha de baixo para cima, como o chat normal." value={cfg.direction} options={DIRECTIONS} onChange={v=>set({direction:v})}/>
      <RangeRow label="Máximo de mensagens" hint="Quantas mensagens ficam na coluna antes de sair a mais antiga." min={10} max={300} step={5} value={cfg.max} fmt={v=>String(v)} onChange={v=>set({max:v})}/>
      <RangeRow label="Some depois de" hint="Tempo até a mensagem sumir da tela; nunca mantém sempre." min={0} max={120} step={5} value={cfg.hideAfter} fmt={v=>v===0?'nunca':v+' s'} onChange={v=>set({hideAfter:v})}/>
      <Seg label="Entrada das mensagens" hint="Como a mensagem aparece ao entrar." value={cfg.enter} options={ENTERS} onChange={v=>set({enter:v})}/>
      <RangeRow label="Largura da coluna" hint="Largura da coluna dentro da cena 1920×1080." min={240} max={960} step={20} value={cfg.width} fmt={v=>v+'px'} onChange={v=>set({width:v})}/>
      <Seg label="Lado na tela" hint="Qual canto da cena a coluna ocupa." value={cfg.align} options={ALIGNS} onChange={v=>set({align:v})}/>
      <div className="lp-ctrl">
       <ToggleCheck label="Listras alternadas no fundo" hint="Zebra sutil atrás de cada mensagem." checked={cfg.stripes} onChange={v=>set({stripes:v})}/>
      </div>
     </div>

     <div className="lp-group">
      <h4>Explosão de emotes</h4>
      <div className="lp-ctrl">
       <ToggleCheck label="Também nesta camada" hint="A explosão de sub, presente ou meta completada cai também sobre a coluna de chat." checked={burstChat} onChange={v=>void saveBurstChat(v)}/>
      </div>
      <p className="lp-note">Quem dispara, quantos emotes e o tamanho ficam na tela Alertas. Desligado aqui, a explosão toca só nas camadas de alertas e meta.</p>
     </div>

     <div className="lp-group">
      <h4>Mensagem</h4>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Fonte</span>
       <select value={cfg.font} aria-label="Fonte do chat" onChange={e=>set({font:e.target.value})}>
        {fontGroups.map(g=><optgroup key={g.label} label={g.label}>{g.fonts.map(f=><option key={f} value={f}>{f}</option>)}</optgroup>)}
       </select>
       <small className="lp-note">Google Fonts exigem internet no computador do OBS; fontes do sistema funcionam offline.</small>
      </div>
      <RangeRow label="Tamanho do texto" hint="Tamanho das mensagens." min={10} max={28} step={1} value={cfg.fontSize} fmt={v=>v+'px'} onChange={v=>set({fontSize:v})}/>
      <RangeRow label="Tamanho do nome" hint="Tamanho do nome de quem falou." min={10} max={36} step={1} value={cfg.nameSize} fmt={v=>v+'px'} onChange={v=>set({nameSize:v})}/>
      <RangeRow label="Tamanho dos emotes" hint="Percentual sobre o tamanho normal; 100% é o padrão." min={50} max={200} step={5} value={cfg.emoteSize} fmt={v=>v+'%'} onChange={v=>set({emoteSize:v})}/>
      <div className="lp-ctrl">
       <ToggleCheck label="Emote grande em mensagem só de emote" hint="Como na Twitch: 1 emote aparece no maior tamanho (4×), 2 ou 3 em dobro, o resto no normal." checked={cfg.bigEmotes} onChange={v=>set({bigEmotes:v})}/>
      </div>
      <div className="lp-ctrl">
       <ToggleCheck label="Emotes explodem ao aparecer" hint="Os emotes saem voando da mensagem, como na referência da StreamElements; o texto continua no lugar." checked={cfg.emoteFloat} onChange={v=>set({emoteFloat:v})}/>
      </div>
      <ColorRow label="Cor do texto" hint="Cor da mensagem; o nome pode ter cor própria." value={cfg.color} onChange={v=>set({color:v})}/>
      <div className="lp-ctrl">
       <ToggleCheck label="Nome em caixa alta" hint="Deixa o nome todo em maiúsculas." checked={cfg.upper} onChange={v=>set({upper:v})}/>
      </div>
      <Seg label="Origem da cor do nome" hint="Twitch: a cor real de cada pessoa (quem nunca escolheu ganha uma cor estável por conta). Personalizadas: cor estável por espectador. Fixa: uma cor só." value={cfg.nameMode} options={NAMEMODES} onChange={v=>set({nameMode:v})}/>
      {cfg.nameMode==='fixed'&&<ColorRow label="Cor do nome" hint="Uma cor só para todo mundo no chat." value={cfg.nameFixed||'#ffffff'} onChange={v=>set({nameFixed:v})}/>}
      <div className="lp-ctrl">
       <ToggleCheck label="Prefixo antes do nome" hint="O “&gt;” do widget de referência, antes das badges; texto e cor livres." checked={cfg.prefixOn} onChange={v=>set({prefixOn:v})}/>
      </div>
      {cfg.prefixOn&&<>
       <Field label="Texto do prefixo" hint="Até 16 caracteres."><input value={cfg.prefix} maxLength={16} onChange={e=>set({prefix:e.target.value})}/></Field>
       <ColorRow label="Cor do prefixo" value={cfg.prefixColor} onChange={v=>set({prefixColor:v})}/>
      </>}
     </div>

     <div className="lp-group">
      <h4>Badges</h4>
      <p className="lp-note">Qual insígnia aparece ao lado do nome. Desligue as que não quiser exibir.</p>
      <div className="lp-toggles">
       {BADGE_LABELS.map(([k,l])=><ToggleCheck key={k} label={l} checked={cfg.badges[k]} onChange={v=>setCfg(c=>({...c,badges:{...c.badges,[k]:v}}))}/>)}
      </div>
     </div>

     <div className="lp-group">
      <h4>Destaques por papel</h4>
      <Seg label="Como destacar" hint="O que a cor do papel pinta na mensagem." value={cfg.hlMode} options={HLMODES} onChange={v=>set({hlMode:v})}/>
      <div className="lp-colors">
       <ColorRow label="Transmissor" hint="Cor do seu nome no chat." value={cfg.hl.broadcaster} onChange={v=>set({hl:{...cfg.hl,broadcaster:v}})}/>
       <ColorRow label="Moderador" hint="Cor dos moderadores." value={cfg.hl.moderator} onChange={v=>set({hl:{...cfg.hl,moderator:v}})}/>
       <ColorRow label="VIP" hint="Cor dos VIPs." value={cfg.hl.vip} onChange={v=>set({hl:{...cfg.hl,vip:v}})}/>
       <ColorRow label="Inscrito" hint="Cor dos inscritos." value={cfg.hl.subscriber} onChange={v=>set({hl:{...cfg.hl,subscriber:v}})}/>
      </div>
      <div className="lp-ctrl">
       <ToggleCheck label="Destacar a primeira mensagem" hint="“Primeira vez” é desde que o BotLive abriu, não a primeira da vida no canal." checked={cfg.firstOn} onChange={v=>set({firstOn:v})}/>
       {cfg.firstOn&&<ColorRow label="Cor da primeira mensagem" value={cfg.firstColor} onChange={v=>set({firstColor:v})}/>}
      </div>
     </div>

     <div className="lp-group">
      <h4>Eventos na coluna</h4>
      <p className="lp-note">Follow, sub, renovação, presente, bits e raid viram uma caixa própria dentro da coluna — com cor e som por evento. Funciona junto ou separado da camada de alertas.</p>
      <div className="lp-ctrl">
       <ToggleCheck label="Eventos no chat" hint="Desligado, nada muda na coluna — os testes avisam que, em live, também não apareceria." checked={cfg.eventsOn} onChange={v=>set({eventsOn:v})}/>
      </div>
      {cfg.eventsOn&&<>
       <div className="lp-ctrl">
        <ToggleCheck label="Mostrar a mensagem do espectador" hint="Renovação e bits com texto entram como segunda linha da caixa." checked={cfg.evShowMsg} onChange={v=>set({evShowMsg:v})}/>
       </div>
       <div className="al-pills" role="group" aria-label="Eventos do chat">
        {CHAT_EVENT_KINDS.map(k=><button key={k} type="button" className={'lp-seg'+(evSel===k?' selected':'')} aria-pressed={evSel===k} onClick={()=>setEvSel(k)}>
         {CHAT_EVENT_LABELS[k]}{!cfg.ev[k].on&&<span className="al-pill-off" title="Desligado">•</span>}
        </button>)}
       </div>
       <div className="lp-ctrl">
        <ToggleCheck label={'Exibir caixa de '+CHAT_EVENT_LABELS[evSel].toLowerCase()} hint="Desligado, este evento não desenha nada na coluna." checked={cfg.ev[evSel].on} onChange={v=>setEv(evSel,{on:v})}/>
       </div>
       <Field label="Texto da caixa" hint="Marcadores: {name} {amount} {months} {tier} {sender} {message}."><input value={cfg.ev[evSel].text} maxLength={160} onChange={e=>setEv(evSel,{text:e.target.value})}/></Field>
       <ColorRow label="Cor do evento" hint="Pinta a borda e o fundo translúcido da caixa." value={cfg.ev[evSel].color} onChange={v=>setEv(evSel,{color:v})}/>
       <div className="lp-ctrl">
        <span className="lp-ctrl-label">Som e teste</span>
        <div className="lp-segrow">
         <select value={cfg.ev[evSel].sound} aria-label={'Som de '+CHAT_EVENT_LABELS[evSel]} onChange={e=>setEv(evSel,{sound:e.target.value})}>
          <option value="">Sem som</option>
          {sounds.map(s=><option key={s.id} value={s.id}>{s.name}</option>)}
          {!sounds.some(s=>s.id===cfg.ev[evSel].sound)&&<MissingOption value={cfg.ev[evSel].sound} label="Som salvo indisponível — escolha outro"/>}
         </select>
         <button type="button" className="lp-seg" onClick={()=>void testEvent(evSel)}><Play size={12}/> Testar {CHAT_EVENT_LABELS[evSel].toLowerCase()}</button>
        </div>
        <small className="lp-note">{sounds.length?(desktop?'O som sai pela saída única do BotLive (tela Áudio no OBS), nunca dentro do OBS. Envie áudios na tela Sons de Live.':'Na prévia web só o visual toca: no aplicativo o som sai pela saída única (tela Áudio no OBS). Envie áudios na tela Sons de Live.'):'Nenhum som neste perfil ainda — envie um WAV, MP3 ou OGG na tela Sons de Live.'}</small>
       </div>
      </>}
     </div>

     <div className="lp-group">
      <h4>Filtros</h4>
      <div className="lp-ctrl">
       <ToggleCheck label="Mostrar as mensagens do bot" hint="Inclui o próprio bot na coluna." checked={cfg.showBot} onChange={v=>set({showBot:v})}/>
       <Field label="Nome do bot no chat" hint="Como o bot aparece no chat da plataforma."><input value={cfg.botName} maxLength={64} onChange={e=>set({botName:e.target.value})}/></Field>
      </div>
      <div className="lp-ctrl">
       <ToggleCheck label="Esconder outros bots" hint="Some com robôs que falam no chat." checked={cfg.hideBots} onChange={v=>set({hideBots:v})}/>
       <Field label="Lista de bots" hint="Nomes separados por vírgula; nomes terminados em “bot” já são escondidos."><input value={cfg.bots} maxLength={200} onChange={e=>set({bots:e.target.value})}/></Field>
      </div>
      <div className="lp-ctrl">
       <ToggleCheck label="Esconder comandos (!)" hint="Mensagens que começam com ! ou / não aparecem." checked={cfg.hideCommands} onChange={v=>set({hideCommands:v})}/>
      </div>
      <Field label="Palavras bloqueadas" hint="Separadas por vírgula; a mensagem que contiver alguma delas não aparece."><input value={cfg.blocked} maxLength={200} onChange={e=>set({blocked:e.target.value})}/></Field>
     </div>
     </CodeTabs>
    </div>
   </div>
  </Card>

  <Card title="No OBS" action={info?<span className="status-pill on"><span className="status-dot" aria-hidden="true"></span>Pronto</span>:undefined}>
   {info?<>
    <p className="help">Este é o <strong>endereço da camada de chat</strong> (o parâmetro <code>mode=chat</code> deixa só o chat nesta fonte). Cada camada — meta, alertas e chat — fica na sua própria Fonte de navegador. Coloque a coluna onde o chat não cubra a cena — o alinhamento acima define o lado.</p>
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

/** Grupo de botões lado a lado. */
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

function ColorRow({label,hint,value,onChange}:{label:string;hint?:string;value:string;onChange:(v:string)=>void}){
 const rgb=/^#[0-9a-fA-F]{6}$/.test(value)?value:'#101721';
 return <div className="lp-color">
  <span>{label}</span>
  <input type="color" value={rgb} aria-label={label} onChange={e=>onChange(e.target.value)}/>
  <code aria-hidden="true">{rgb.toUpperCase()}</code>
  {hint&&<small className="lp-note">{hint}</small>}
 </div>;
}
