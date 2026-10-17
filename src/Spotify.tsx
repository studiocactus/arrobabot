import {useEffect, useRef, useState} from 'react';
import {listen} from '@tauri-apps/api/event';
import {Check, Copy, ExternalLink, Plug, Unplug} from 'lucide-react';
import {api, desktop, errorText} from './api';
import {Card, Field, Toggle} from './components';
import {
 DEFAULT_SPOTIFY, SPOTIFY_DASHBOARD, SPOTIFY_REDIRECT,
 elapsed, fmtTime, normalizeNow, normalizeSpotify, type NowTrack, type SpotifyConfig,
} from './spotifyOverlay';
import './overlay-studio.css';

type GetResult={config:unknown;connected:boolean;now:unknown};
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

export default function SpotifyScreen({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [cfg,setCfg]=useState<SpotifyConfig|null>(null);
 const [connected,setConnected]=useState(false);
 const [account,setAccount]=useState('');
 const [now,setNow]=useState<NowTrack|null>(null);
 const [saveState,setSaveState]=useState<'idle'|'saving'|'saved'|'error'>('idle');
 const [connecting,setConnecting]=useState(false);
 const [polling,setPolling]=useState(false);
 const [copied,setCopied]=useState('');
 const hydrated=useRef(false);const lastSaved=useRef('');

 useEffect(()=>{
  hydrated.current=false;
  api<GetResult>('spotify.get',{profileId}).then(r=>{
   setCfg(normalizeSpotify(r.config));setConnected(r.connected);setNow(normalizeNow(r.now));
   lastSaved.current=JSON.stringify(normalizeSpotify(r.config));hydrated.current=true;
  }).catch(e=>notify(errorText(e)));
 },[profileId,notify]);

 // A música chega pelo mesmo caminho do overlay — a linha acompanha a live.
 useEffect(()=>{
  const un=listen<{profileId?:string}>('spotify.now',ev=>{
   if(ev.payload?.profileId===profileId)setNow(normalizeNow(ev.payload));
  });
  return()=>{un.then(f=>f())};
 },[profileId]);

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
 function copy(k:string){
  navigator.clipboard.writeText(`{{${k}}}`).then(()=>{setCopied(k);setTimeout(()=>setCopied(''),1800)}).catch(()=>{});
 }

 if(!cfg)return <p className="help">Carregando…</p>;
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
   <p className="help">Com o interruptor ligado, o BotLive consulta o Spotify de 5 em 5 segundos e transmite a faixa para o overlay e as automações.</p>
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
