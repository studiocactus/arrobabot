import {useEffect,useRef,useState,type ReactNode} from 'react';
import {listen} from '@tauri-apps/api/event';
import {Check, Copy, ExternalLink, Monitor, Play, Plug, Unplug} from 'lucide-react';
import {api, desktop, errorText} from './api';
import {Card, Check as ToggleCheck, Field, Toggle} from './components';
import {ensureFont} from './overlayDesign';
import {
 DEFAULT_SPOTIFY, SPOTIFY_COVERS, SPOTIFY_DASHBOARD, SPOTIFY_FONTS, SPOTIFY_LAYOUTS,
 SPOTIFY_POSITIONS, SPOTIFY_REDIRECT, SPOTIFY_TRANSITIONS,
 elapsed, fmtTime, normalizeNow, normalizeSpotify, spotifyPlayer,
 type NowTrack, type SpotifyConfig, type SpotifyCover, type SpotifyDesign,
 type SpotifyLayout, type SpotifyPlayer, type SpotifyPosition, type SpotifyTransition,
} from './spotifyOverlay';
import './overlay-studio.css';

type OverlayInfo={token:string;port:number;url:string};
type GetResult={config:unknown;connected:boolean;now:unknown;overlay?:OverlayInfo};
type ConnectResult={authorized:boolean;user?:string;product?:string};

/** Variáveis que a música oferece às automações (mesmas do backend). */
const NOW_VARS:[string,string][]=[
 ['nowplaying.musica','Música tocando'],
 ['nowplaying.artista','Artista'],
 ['nowplaying.album','Álbum'],
 ['nowplaying.tempo','Posição na música'],
 ['nowplaying.duracao','Duração'],
 ['nowplaying.link','Link da música'],
];

/** Peles do cartão em miniatura — o desenho que cada uma escolhe. */
const SKIN_ART:Record<SpotifyLayout,ReactNode>={
 card:<svg viewBox="0 0 34 22" width="34" height="22" aria-hidden="true"><rect x="1" y="4" width="14" height="14" rx="2.5" fill="currentColor" opacity=".45"/><rect x="18" y="6" width="14" height="3" rx="1.5" fill="currentColor"/><rect x="18" y="11" width="9" height="2.5" rx="1.25" fill="currentColor" opacity=".6"/><rect x="18" y="15.5" width="14" height="2.5" rx="1.25" fill="currentColor" opacity=".35"/></svg>,
 stacked:<svg viewBox="0 0 34 22" width="34" height="22" aria-hidden="true"><rect x="10" y="1" width="14" height="10" rx="2.5" fill="currentColor" opacity=".45"/><rect x="5" y="13.5" width="24" height="3" rx="1.5" fill="currentColor"/><rect x="9" y="18.5" width="16" height="2" rx="1" fill="currentColor" opacity=".4"/></svg>,
 compact:<svg viewBox="0 0 34 22" width="34" height="22" aria-hidden="true"><rect x="1" y="4" width="10" height="10" rx="2" fill="currentColor" opacity=".45"/><rect x="14" y="5.5" width="18" height="3" rx="1.5" fill="currentColor"/><rect x="14" y="10.5" width="11" height="2.5" rx="1.25" fill="currentColor" opacity=".6"/><rect x="1" y="17" width="31" height="2.5" rx="1.25" fill="currentColor" opacity=".3"/></svg>,
 minimal:<svg viewBox="0 0 34 22" width="34" height="22" aria-hidden="true"><rect x="1" y="5" width="26" height="3.5" rx="1.75" fill="currentColor"/><rect x="1" y="11" width="16" height="3" rx="1.5" fill="currentColor" opacity=".6"/><rect x="1" y="17" width="31" height="2.5" rx="1.25" fill="currentColor" opacity=".3"/></svg>,
};

/** Temas prontos: um clique pinta o cartão inteiro; as cores ficam soltas para ajuste fino. */
const THEMES:{label:string;colors:Record<string,string>}[]=[
 {label:'Escuro',colors:{bg:'#0f1319',titleColor:'#ffffff',artistColor:'#b9c0cc',albumColor:'#8a92a3',barBg:'#2a3140'}},
 {label:'Claro',colors:{bg:'#f4f6fa',titleColor:'#14181f',artistColor:'#4b5563',albumColor:'#6b7280',barBg:'#dbe1ea'}},
];
/** Amostras de cor de destaque — a cor exata fica no seletor de Cores. */
const TINTS=['#1ed760','#38b6ff','#2dd4bf','#a3e635','#facc15','#fb923c','#f87171','#e879f9','#a78bfa'];

/** Fundos da prévia: como o cartão fica sobre cada cena. */
type FrameBg='dark'|'checker'|'light';
const BG_LABELS:Record<FrameBg,string>={dark:'escuro',checker:'xeque',light:'claro'};

/** Amostra que a prévia desenha quando a live não está com música. */
function sampleTrack():NowTrack{
 return{id:'sample',name:'Tocando Agora',artist:'Cactus, convidada',album:'Ao Vivo',art:'',url:'',
  durationMs:201000,progressMs:83250,timestamp:Date.now(),playing:true};
}

/**
 * Estúdio da música: o mesmo cartão que o OBS desenha, ajustável ao vivo.
 * A prévia roda o módulo compartilhado da camada `spotify` — o que se vê aqui
 * é o que a Fonte de navegador mostra, e o teste segue o caminho do poll real.
 */
export default function SpotifyScreen({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [cfg,setCfg]=useState<SpotifyConfig|null>(null);
 const [connected,setConnected]=useState(false);
 const [account,setAccount]=useState('');
 const [now,setNow]=useState<NowTrack|null>(null);
 const [info,setInfo]=useState<OverlayInfo|null>(null);
 const [saveState,setSaveState]=useState<'idle'|'saving'|'saved'|'error'>('idle');
 const [connecting,setConnecting]=useState(false);
 const [polling,setPolling]=useState(false);
 const [testing,setTesting]=useState(false);
 const [copied,setCopied]=useState('');
 const [ensaio,setEnsaio]=useState(false);
 const [previewEmpty,setPreviewEmpty]=useState(false);
 const [bg,setBg]=useState<FrameBg>('dark');
 const [scale,setScale]=useState(0.4);
 const hydrated=useRef(false);const lastSaved=useRef('');
 const frame=useRef<HTMLDivElement|null>(null);
 const hostRef=useRef<HTMLDivElement|null>(null);
 const playerRef=useRef<SpotifyPlayer|null>(null);

 useEffect(()=>{
  hydrated.current=false;
  api<GetResult>('spotify.get',{profileId}).then(r=>{
   setCfg(normalizeSpotify(r.config));setConnected(r.connected);setNow(normalizeNow(r.now));
   setInfo(r.overlay&&r.overlay.token?r.overlay:null);
   lastSaved.current=JSON.stringify(normalizeSpotify(r.config));hydrated.current=true;
  }).catch(e=>notify(errorText(e)));
 },[profileId,notify]);

 // A música chega pelo mesmo caminho do overlay — a linha e a prévia acompanham a live.
 useEffect(()=>{
  const un=listen<{profileId?:string;rehearsal?:boolean}>('spotify.now',ev=>{
   if(ev.payload?.profileId!==profileId)return;
   if(ev.payload.rehearsal===true){
    const t=normalizeNow(ev.payload);
    if(t)playerRef.current?.sample(t,6000);
    setEnsaio(true);setTimeout(()=>setEnsaio(false),6000);
    return;
   }
   setNow(normalizeNow(ev.payload));
  });
  return()=>{un.then(f=>f())};
 },[profileId]);

 // Desenho da prévia: o mesmo código da fonte do OBS, criado junto do quadro
 // (o host só existe depois que a config carrega — por isso no mesmo efeito).
 useEffect(()=>{
  if(!cfg||!hostRef.current)return;
  if(!playerRef.current)playerRef.current=spotifyPlayer(hostRef.current);
  ensureFont(cfg.design.font);
  const p=playerRef.current;
  if(now)p.update(cfg.design,now);
  else if(previewEmpty)p.update(cfg.design,null);
  else p.update(cfg.design,sampleTrack());
 },[cfg,now,previewEmpty]);
 useEffect(()=>()=>{playerRef.current?.destroy();playerRef.current=null},[]);

 // Quadro de 1920×1080 em miniatura, como nos outros Estúdios.
 useEffect(()=>{
  const el=frame.current;if(!el)return;
  const ro=new ResizeObserver(()=>setScale(el.clientWidth/1920));
  ro.observe(el);setScale(el.clientWidth/1920);
  return()=>ro.disconnect();
 },[]);

 // Autosave curto: o backend valida e a prévia/OBS retransmitem na hora.
 useEffect(()=>{
  if(!desktop||!hydrated.current||!cfg)return;
  const key=JSON.stringify(cfg);
  if(key===lastSaved.current)return;
  setSaveState('saving');
  let cancelled=false;
  const timer=setTimeout(async()=>{
   try{
    await api('module.config',{profileId,key:'spotify',value:cfg});
    lastSaved.current=key;
    if(!cancelled)setSaveState('saved');
   }catch(e){if(!cancelled)setSaveState('error')}
  },700);
  return()=>{cancelled=true;clearTimeout(timer)};
 },[cfg,profileId,desktop]);

 async function authorize(){
  if(!cfg)return;setConnecting(true);
  try{
   const r=await api<ConnectResult>('spotify.connect',{profileId});
   setConnected(r.authorized);setAccount(r.user||'');
   notify(r.authorized?'Conta do Spotify autorizada':'Autorização cancelada');
  }catch(e){notify(errorText(e))}finally{setConnecting(false)}
 }
 async function disconnect(){
  try{await api('spotify.disconnect',{profileId});setConnected(false);setAccount('');setNow(null);notify('Spotify desconectado')}
  catch(e){notify(errorText(e))}
 }
 async function testPoll(){
  if(!cfg)return;setPolling(true);
  try{
   const r=await api<{track:unknown}>('spotify.poll',{profileId});
   const t=normalizeNow(r.track);
   setNow(t);setConnected(true);
   notify(t?`Lido: ${t.name} — ${t.artist}`:'O Spotify não tem música tocando agora');
  }catch(e){notify(errorText(e))}finally{setPolling(false)}
 }
 async function testOverlay(){
  setTesting(true);
  try{
   await api('spotify.test',{profileId});
   notify('Amostra disparada — o OBS mostra com o selo Ensaio.');
  }catch(e){notify(errorText(e))}finally{setTesting(false)}
 }
 const copyUrl=async()=>{
  if(!info)return;
  try{
   await navigator.clipboard.writeText(layerUrl());
   setCopied('url');setTimeout(()=>setCopied(''),2200);
   notify('Endereço da camada de música copiado.');
  }catch{notify('Não foi possível copiar. Selecione o endereço e copie manualmente.')}
 };
 /** Endereço da camada de música: mesma base do overlay com mode=spotify. */
 const layerUrl=()=>info?info.url+'&mode=spotify':'';
 function copy(k:string){
  navigator.clipboard.writeText(`{{${k}}}`).then(()=>{setCopied(k);setTimeout(()=>setCopied(''),1800)}).catch(()=>{});
 }
 const d=(patch:Partial<SpotifyDesign>)=>setCfg(c=>c?{...c,design:{...c.design,...patch}}:c);
 const cycleBg=()=>setBg(b=>b==='dark'?'checker':b==='checker'?'light':'dark');

 if(!cfg)return <p className="help">Carregando…</p>;
 const design=cfg.design;
 const chip=ensaio?{cls:'ensaio',label:'Ensaio'}:now?{cls:'live',label:'Ao vivo'}:previewEmpty?{cls:'',label:'Parado'}:{cls:'',label:'Amostra'};
 return <>
  <Card title="Conexão com o Spotify" action={connected?<span className="tag green">Conectado{account?` — ${account}`:''}</span>:<span className="tag">Não autorizado</span>}>
   <p className="help">
    O Spotify só entrega a música para aplicativos autorizados. Crie o seu aplicativo gratuito (leva 2 minutos), copie o Client ID para cá e autorize a sua conta — é o mesmo caminho de qualquer app de terceiros, e os dados ficam só na sua máquina.
   </p>
   <div className="form-grid">
    <Field label="Client ID do seu aplicativo" hint="No painel do Spotify: Criar app → Web API → Settings → Client ID.">
     <input value={cfg.clientId} placeholder="3f5b…" onChange={e=>setCfg({...cfg,clientId:e.target.value})}/>
    </Field>
    <Field label="URL de retorno (cadastre no app do Spotify)" hint="Em Redirect URIs, do mesmo jeito que o Client ID.">
     <div className="lp-copyrow">
      <input readOnly value={SPOTIFY_REDIRECT}/>
      <button className="ghost" onClick={()=>copy('redirect')} title="Copiar URL de retorno">{copied==='redirect'?<Check size={16}/>:<Copy size={16}/>}</button>
     </div>
    </Field>
   </div>
   <div className="form-footer">
    <div className="lp-actions">
     <button disabled={!cfg.clientId||connecting} onClick={authorize}>{connecting?'Aguarde o navegador…':'Autorizar conta'}</button>
     <button className="ghost" disabled={!connected||connecting} onClick={disconnect}><Unplug size={16}/> Desconectar</button>
     <a className="text-button" href={SPOTIFY_DASHBOARD} target="_blank" rel="noreferrer">Painel do Spotify<ExternalLink size={14}/></a>
    </div>
    <span className={'lp-save lp-save--'+saveState}>{saveState==='saving'?'Salvando…':saveState==='saved'?'Salvo':saveState==='error'?'Não salvou':'Alterações salvam sozinhas'}</span>
   </div>
  </Card>

  <Card title="Música no overlay">
   <Toggle checked={cfg.enabled} onChange={v=>setCfg({...cfg,enabled:v})} label="Ler a música que está tocando"/>
   <p className="help">Com o interruptor ligado, o BotLive consulta o Spotify de 5 em 5 segundos e transmite a faixa para o overlay, a prévia e as automações.</p>
   <div className="lp-nowplaying">
    <span className="lp-nowplaying-label">Agora tocando</span>
    {now?<strong>{now.name} — {now.artist} <em>({fmtTime(elapsed(now))}/{fmtTime(now.durationMs)})</em>{now.playing?'':' · pausado'}</strong>:<span className="help">Nada tocando agora.</span>}
   </div>
   <div className="form-footer">
    <div className="lp-actions">
     <button className="ghost" disabled={polling||connecting} onClick={testPoll}><Plug size={16}/> {polling?'Consultando…':'Testar conexão'}</button>
    </div>
   </div>
  </Card>

  <Card title="Estúdio da música" action={<span className={'lp-save lp-save--'+saveState}>{saveState==='saving'?'Salvando…':saveState==='saved'?'Salvo':saveState==='error'?'Não salvou':'Alterações salvam sozinhas'}</span>}>
   <p className="help">A prévia desenha o mesmo cartão que o OBS recebe. Mexa no design à direita e ele muda na hora; o teste dispara a amostra no OBS com o selo Ensaio, pelo mesmo caminho da música real.</p>

   <div className="lp-studio">
    <div className="lp-preview">
     <div className="lp-stagebar">
      <span className={'lp-chip '+chip.cls}>{chip.label}</span>
      <div className="lp-segrow" role="group" aria-label="Prévia">
       <button type="button" className={'lp-seg'+(!previewEmpty?' selected':'')} aria-pressed={!previewEmpty} disabled={!!now} onClick={()=>setPreviewEmpty(false)}>Com música</button>
       <button type="button" className={'lp-seg'+(previewEmpty?' selected':'')} aria-pressed={previewEmpty} disabled={!!now} onClick={()=>setPreviewEmpty(true)}>Sem música</button>
      </div>
      <button type="button" className="lp-seg" disabled={testing} onClick={()=>void testOverlay()}><Play size={13}/> Testar no overlay</button>
     </div>

     <div className="lp-frame" data-bg={bg} ref={frame}>
      {ensaio&&<span className="lp-ensaio">Ensaio</span>}
      <div className="lp-canvas" style={{transform:`scale(${scale})`}}>
       <div className="bl-spy-host" ref={hostRef}></div>
      </div>
     </div>

     <div className="lp-stagebar">
      <span className="help">Prévia do quadro de 1920×1080 em escala de {Math.round(scale*100)}%.</span>
      <button type="button" className="lp-seg" onClick={cycleBg}>Fundo: {BG_LABELS[bg]}</button>
     </div>
     {!cfg.enabled&&<p className="help">A música está desligada: o OBS só desenha quando você ativar o interruptor acima — o teste avisa o que falta ligar.</p>}
    </div>

    <div className="lp-controls">
     <Card title="Aparência">
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Pele do cartão</span>
       <div className="sp-skins" role="group" aria-label="Pele do cartão">
        {SPOTIFY_LAYOUTS.map(([v,l])=><button key={v} type="button" className={'sp-skin'+(design.layout===v?' selected':'')} aria-pressed={design.layout===v} onClick={()=>d({layout:v})}>
         {SKIN_ART[v]}<span>{l}</span>
        </button>)}
       </div>
       <small className="lp-note">Só texto dispensa a capa por inteiro.</small>
      </div>
      <Seg label="Posição na tela" hint="Canto do quadro de 1920×1080; o cartão fica a 18 px da borda." value={design.position} options={SPOTIFY_POSITIONS} onChange={v=>d({position:v})}/>
      <RangeRow label="Largura" min={240} max={960} step={10} value={design.width} fmt={v=>v+'px'} onChange={v=>d({width:v})}/>
      <RangeRow label="Preenchimento" min={0} max={48} step={1} value={design.padding} fmt={v=>v+'px'} onChange={v=>d({padding:v})}/>
      <RangeRow label="Cantos" min={0} max={40} step={1} value={design.radius} fmt={v=>v+'px'} onChange={v=>d({radius:v})}/>
     </Card>

     <Card title="Capa">
      <Toggle checked={design.showArt} onChange={v=>d({showArt:v})} label="Mostrar a capa do álbum"/>
      <Seg label="Formato da capa" value={design.cover} options={SPOTIFY_COVERS} onChange={v=>d({cover:v})}/>
      <Toggle checked={design.coverGlow} onChange={v=>d({coverGlow:v})} label="Brilho na capa"/>
      <ToggleCheck label="Capa desfocada ao fundo" hint="A própria capa vira um fundo borrado atrás do cartão; dispensa quando não há capa." checked={design.coverBlur} onChange={v=>d({coverBlur:v})}/>
      <RangeRow label="Tamanho da capa" min={48} max={220} step={2} value={design.artSize} fmt={v=>v+'px'} onChange={v=>d({artSize:v})}/>
      <RangeRow label="Cantos da capa" hint="No formato redondo o canto é a própria bola." min={0} max={60} step={1} value={design.artRadius} fmt={v=>v+'px'} onChange={v=>d({artRadius:v})}/>
     </Card>

     <Card title="Campos">
      <Toggle checked={design.showAlbum} onChange={v=>d({showAlbum:v})} label="Linha do álbum"/>
      <Toggle checked={design.showProgress} onChange={v=>d({showProgress:v})} label="Barra de progresso"/>
      <Toggle checked={design.showTimes} onChange={v=>d({showTimes:v})} label="Tempos (posição e duração)"/>
     </Card>

     <Card title="Cores">
      <Seg label="Tema" hint="Um clique pinta o cartão inteiro; ajuste qualquer cor em seguida." value={(THEMES.find(t=>t.colors.bg===design.bg)?.label)||''} options={THEMES.map(t=>[t.label,t.label] as [string,string])} onChange={l=>{const t=THEMES.find(x=>x.label===l);if(t)d(t.colors as Partial<SpotifyDesign>)}}/>
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Cor de destaque</span>
       <div className="sp-tints" role="group" aria-label="Cor de destaque">
        {TINTS.map(c=><button key={c} type="button" className={'sp-tint'+(design.barColor.toLowerCase()===c?' selected':'')} style={{background:c}} aria-label={`Destaque ${c}`} aria-pressed={design.barColor.toLowerCase()===c} onClick={()=>d({barColor:c})}/>)}
       </div>
       <small className="lp-note">Amostras rápidas; a cor exata fica no seletor da barra, abaixo.</small>
      </div>
      <ColorRow label="Fundo do cartão" value={design.bg} onChange={v=>d({bg:v})}/>
      <ColorRow label="Cor do título" value={design.titleColor} onChange={v=>d({titleColor:v})}/>
      <ColorRow label="Cor do artista" value={design.artistColor} onChange={v=>d({artistColor:v})}/>
      <ColorRow label="Cor do álbum" value={design.albumColor} onChange={v=>d({albumColor:v})}/>
      <ColorRow label="Cor da barra" hint="Também é a cor do brilho da capa." value={design.barColor} onChange={v=>d({barColor:v})}/>
      <ColorRow label="Cor de trás da barra" value={design.barBg} onChange={v=>d({barBg:v})}/>
     </Card>

     <Card title="Tipografia">
      <div className="lp-ctrl">
       <span className="lp-ctrl-label">Fonte</span>
       <select value={design.font} aria-label="Fonte" onChange={e=>d({font:e.target.value})}>
        {SPOTIFY_FONTS.map(f=><option key={f} value={f}>{f}</option>)}
       </select>
      </div>
      <RangeRow label="Tamanho do título" min={10} max={48} step={1} value={design.titleSize} fmt={v=>v+'px'} onChange={v=>d({titleSize:v})}/>
      <RangeRow label="Tamanho do artista" min={9} max={32} step={1} value={design.artistSize} fmt={v=>v+'px'} onChange={v=>d({artistSize:v})}/>
     </Card>

     <Card title="Barra de progresso">
      <RangeRow label="Altura da barra" min={2} max={24} step={1} value={design.barHeight} fmt={v=>v+'px'} onChange={v=>d({barHeight:v})}/>
      <RangeRow label="Cantos da barra" min={0} max={12} step={1} value={design.barRadius} fmt={v=>v+'px'} onChange={v=>d({barRadius:v})}/>
     </Card>

     <Card title="Movimento">
      <Seg label="Entrar" hint="Como o cartão aparece quando a música começa." value={design.transition} options={SPOTIFY_TRANSITIONS} onChange={v=>d({transition:v})}/>
      <Seg label="Sair" hint="Como o cartão some quando para ou some do overlay." value={design.exitTransition} options={SPOTIFY_TRANSITIONS} onChange={v=>d({exitTransition:v})}/>
      <RangeRow label="Duração da animação" hint="Vale para entrar e para sair." min={0} max={2000} step={50} value={design.transitionMs} fmt={v=>v+' ms'} onChange={v=>d({transitionMs:v})}/>
     </Card>

     <Card title="Quando a música para">
      <Seg label="Sem música tocando" value={design.empty} options={[['hide','Esconder o cartão'],['message','Mostrar mensagem']] as [SpotifyConfig['design']['empty'],string][]} onChange={v=>d({empty:v})}/>
      <Field label="Mensagem do título" hint="Aparece no lugar do nome da música.">
       <input value={design.emptyText} maxLength={64} onChange={e=>d({emptyText:e.target.value})}/>
      </Field>
      <Field label="Mensagem do artista" hint="Segunda linha do estado parado; vazio dispensa.">
       <input value={design.emptyArtist} maxLength={64} placeholder="Sem linha extra" onChange={e=>d({emptyArtist:e.target.value})}/>
      </Field>
      <Toggle checked={design.hideOnPause} onChange={v=>d({hideOnPause:v})} label="Esconder ao pausar"/>
      <RangeRow label="Espera antes de esconder" hint="Segundos entre a pausa e o cartão sumir; 0 é na hora." disabled={!design.hideOnPause} min={0} max={30} step={1} value={design.pauseDelay} fmt={v=>v===0?'na hora':v+' s'} onChange={v=>d({pauseDelay:v})}/>
     </Card>
    </div>
   </div>
  </Card>

  <Card title="No OBS" action={info?<span className="status-pill on"><span className="status-dot" aria-hidden="true"></span>Pronto</span>:undefined}>
   {info?<>
    <p className="help">Este é o <strong>endereço da camada de música</strong> (o parâmetro <code>mode=spotify</code> deixa só o cartão da música nesta fonte). Cada camada — meta, alertas, chat e música — fica na sua própria Fonte de navegador, com posição e tamanho independentes.</p>
    <div className="lp-url">
     <code>{layerUrl()}</code>
     <button className="primary" onClick={()=>void copyUrl()}>{copied==='url'?<Check size={15}/>:<Copy size={15}/>}{copied==='url'?'Copiado':'Copiar endereço'}</button>
    </div>
    <ol className="lp-steps">
     <li>No OBS: <strong>Fontes → + → Navegador</strong>.</li>
     <li>Cole o endereço acima (o <code>#</code> traz a chave, o perfil e a camada).</li>
     <li>Defina <strong>Largura 1920</strong> e <strong>Altura 1080</strong>.</li>
     <li>Deixe <strong>Atualizar navegador quando a cena ficar ativa</strong> desmarcado: o overlay reconecta sozinho.</li>
    </ol>
    <p className="help">As alterações do Estúdio chegam ao OBS na hora, sem recarregar a fonte.</p>
   </>:<p className="help">O endereço aparece assim que o app estiver rodando no desktop — ele carrega a chave do overlay do perfil.</p>}
  </Card>

  <Card title="Nas automações">
   <p className="help">Use em qualquer automação, resposta automática ou prévia de IA — vazio quando nada toca.</p>
   <ul className="lp-varlist">
    {NOW_VARS.map(([k,label])=><li key={k}>
     <span>{label}</span>
     <code>{`{{${k}}}`}</code>
     <button className="ghost" onClick={()=>copy(k)} title="Copiar variável">{copied===k?<Check size={14}/>:<Copy size={14}/>}</button>
    </li>)}
   </ul>
  </Card>
 </>;
}

/** Grupo de botões lado a lado (pele, posição, tema). Mesmo padrão dos outros Estúdios. */
function Seg<T extends string>({label,hint,value,options,onChange}:{label:string;hint?:string;value:T;options:[T,string][];onChange:(v:T)=>void}){
 return <div className="lp-ctrl">
  <span className="lp-ctrl-label">{label}</span>
  <div className="lp-segrow" role="group" aria-label={label}>
   {options.map(([v,l])=><button key={v} type="button" className={'lp-seg'+(value===v?' selected':'')} aria-pressed={value===v} onClick={()=>onChange(v)}>{l}</button>)}
  </div>
  {hint&&<small className="lp-note">{hint}</small>}
 </div>;
}

function RangeRow({label,hint,min,max,step,value,fmt,disabled,onChange}:{label:string;hint?:string;min:number;max:number;step:number;value:number;fmt:(v:number)=>string;disabled?:boolean;onChange:(v:number)=>void}){
 return <div className="lp-range">
  <span className="lp-range-label">{label}<output>{fmt(value)}</output></span>
  <input type="range" min={min} max={max} step={step} value={value} aria-label={label} disabled={disabled} onChange={e=>onChange(Number(e.target.value))}/>
  {hint&&<small className="lp-note">{hint}</small>}
 </div>;
}

function ColorRow({label,hint,value,onChange}:{label:string;hint?:string;value:string;onChange:(v:string)=>void}){
 const rgb=/^#[0-9a-fA-F]{6}$/.test(value)?value:'#101721';
 return <div className="lp-color">
  <span>{label}</span>
  <input type="color" value={rgb} aria-label={label} onChange={e=>onChange(e.target.value)}/>
  {hint&&<small className="lp-note">{hint}</small>}
 </div>;
}
