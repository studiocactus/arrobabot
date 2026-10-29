import Dashboard from './Dashboard';
import ExecutionHistory from './ExecutionHistory';
import {version} from '../package.json';
import {playViewerSound,stopViewerSounds} from './viewerSound';
import {applyAccent} from './appearance';
import {restoreListen} from './listen';
import {useState,useEffect,useCallback} from 'react';
import {listen} from '@tauri-apps/api/event';
import {LayoutDashboard,Bot,Terminal,Clock,Hash,Workflow,Sparkles,BookOpen,Package,Settings as SettingsIcon,Users,Activity,BarChart3,Plus,ArrowUpRight,ArrowRight,Radio,Zap,Check,ChevronDown,Search,Sun,Moon,Power,Trash2,X,Menu,ShieldCheck,MessageSquare,MessagesSquare,Monitor,Volume2,Clapperboard,Mic,Bell,MessageCircle,Headphones,Music} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {type Snapshot,type Profile,type Flow,type Activity as Log,platforms,fmtTime} from './types';
import {Card,Empty,Modal,Tag,Jump,ScreenBoundary} from './components';
import ProfileForm from './ProfileForm';
import Commands from './Commands';
import Timers from './Timers';
import Counters from './Counters';
import {AIScreen,MemoryScreen} from './AIMemory';
import Presets from './Presets';
import Community from './Community';
import Discord from './Discord';
import Labels from './Labels';
import Sounds from './Sounds';
import Obs from './Obs';
import Voicemod from './Voicemod';
import VoiceCommandsPage from './VoiceCommandsPage';
import LivePix from './LivePix';
import Alertas from './Alertas';
import ChatObs from './ChatObs';
import AudioHub from './AudioHub';
import ChatExtras from './ChatExtras';
import ChatPresence from './ChatPresence';
import Settings from './Settings';
import {Login} from './Access';
import Stats from './Stats';
import SpotifyScreen from './Spotify';
import {navigationAllowed} from './useUnsavedChanges';
import {navigationSections,navigationMatches} from './navigation';
const navItems=[{id:'dashboard',label:'Visão geral',icon:LayoutDashboard,group:'SEU ESPAÇO'},{id:'profiles',label:'Perfis de bot',icon:Bot},{id:'discord',label:'Discord',icon:MessagesSquare},{id:'labels',label:'Rótulos OBS',icon:Monitor},{id:'commands',label:'Comandos',icon:Terminal},{id:'timers',label:'Timers',icon:Clock},{id:'responses',label:'Respostas',icon:MessageSquare},{id:'sounds',label:'Sons de Live',icon:Volume2},{id:'counters',label:'Contadores',icon:Hash},{id:'flows',label:'Automações',icon:Workflow},{id:'obs',label:'OBS Studio',icon:Clapperboard},{id:'audio',label:'Áudio no OBS',icon:Headphones},{id:'vozfx',label:'Voicemod',icon:Mic},{id:'livepix',label:'Meta',icon:Radio,group:'OVERLAYS DO OBS'},{id:'alertas',label:'Alertas',icon:Bell},{id:'chatobs',label:'Chat no OBS',icon:MessageCircle},{id:'spotify',label:'Spotify',icon:Music},{id:'ai',label:'Inteligência artificial',icon:Sparkles,group:'PERSONALIDADE & MEMÓRIA'},{id:'memory',label:'Memórias',icon:BookOpen},{id:'community',label:'Comunidade',icon:Users,group:'FERRAMENTAS'},{id:'presets',label:'Biblioteca de presets',icon:Package},{id:'history',label:'Histórico',icon:Activity},{id:'stats',label:'Estatísticas',icon:BarChart3}];
const nav=navigationSections.flatMap(section=>section.pages.map((id,index)=>({...navItems.find(item=>item.id===id)!,group:index===0?section.label:undefined})));
const subtitles:Record<string,string>={dashboard:'Sua transmissão, audiência e serviços em um só lugar.',profiles:'Cada canal, uma personalidade. Tudo no seu controle.',commands:'Respostas rápidas que aproximam sua comunidade.',timers:'Lembretes no tempo certo, sem esforço.',responses:'Respostas de texto na palavra certa, com som se você quiser.',sounds:'Biblioteca de sons e trilha para cada espectador.',audio:'Uma saída só: todos os sons do bot no mesmo controle do OBS.',obs:'O OBS desta máquina, sem nuvem.',vozfx:'Configure uma vez: recompensas e comandos trocam sua voz no Voicemod.',voicecmds:'Sua fala vira comando: escuta contínua e criação rápida.',livepix:'A meta da sua live, com visual próprio no OBS.',spotify:'A música que está tocando, no seu overlay.',counters:'Cada comando registra o que a comunidade mais usa.',flows:'Conecte ideias. Deixe o bot cuidar do resto.',ai:'Uma voz que combina com a sua comunidade.',memory:'Pequenos detalhes que fazem o bot lembrar.',community:'Mais maneiras de participar da sua live.',discord:'Seu servidor no Discord, moderado junto com a live.',labels:'Arquivos .txt para o OBS e quem está no chat.',presets:'Boas ideias merecem ser compartilhadas.',history:'Entenda o que aconteceu, ação por ação.',stats:'Acompanhe a atividade registrada pelo seu bot.',settings:'Ajuste o BotLive ao seu jeito de fazer live.'};
const blank:Snapshot={profiles:[],logs:[],statuses:{},theme:'dark',apiPort:0,dataDir:''};
export default function App(){
 const [locked,setLocked]=useState(false);
 const [recipeSection,setRecipeSection]=useState<string|undefined>(undefined);
 const [dashboardPrivacy,setDashboardPrivacy]=useState(false);
 const [snapshot,setSnapshot]=useState<Snapshot>(blank);const [page,setPage]=useState('dashboard');const [profileId,setProfileId]=useState('');const [flows,setFlows]=useState<Flow[]>([]);const [toast,setToast]=useState('');const [theme,setTheme]=useState('dark');const [loading,setLoading]=useState(true);const [profileModal,setProfileModal]=useState<Profile|'new'|null>(null);const [deleteProfile,setDeleteProfile]=useState<Profile|null>(null);const [palette,setPalette]=useState(false);const [search,setSearch]=useState('');const [createToken,setCreateToken]=useState(0);const [sidebar,setSidebar]=useState(false);const [historyQuery,setHistoryQuery]=useState('');const [screenNonce,setScreenNonce]=useState(0);
 const notify=useCallback((s:string)=>setToast(s),[]);
 const refresh=useCallback(async()=>{try{const data=await api<Snapshot>('snapshot');setSnapshot(data);applyAccent(data.accent||'');setProfileId(id=>data.profiles.some(p=>p.id===id)?id:data.profiles[0]?.id||'');if(data.theme)setTheme(data.theme)}catch(e){if(errorText(e)==='SESSION_LOCKED')setLocked(true);else notify(errorText(e))}finally{setLoading(false)}},[notify]);
 const profile=snapshot.profiles.find(p=>p.id===profileId);
 const refreshFlows=useCallback(()=>{if(profileId)api<Flow[]>('flows',{profileId}).then(setFlows).catch(e=>notify(errorText(e)));else setFlows([])},[profileId,notify]);
 useEffect(()=>{void refresh()},[refresh]);useEffect(refreshFlows,[refreshFlows]);
 useEffect(()=>{if(desktop&&profileId)void restoreListen(profileId)},[profileId]);
 useEffect(()=>{document.documentElement.dataset.theme=theme},[theme]);
 useEffect(()=>{if(!toast)return;const t=setTimeout(()=>setToast(''),7000);return()=>clearTimeout(t)},[toast]);
 useEffect(()=>{
 const onKey=(e:KeyboardEvent)=>{if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='k'){e.preventDefault();setPalette(v=>!v)}if((e.ctrlKey||e.metaKey)&&e.shiftKey&&e.key.toLowerCase()==='n'){e.preventDefault();if(!navigationAllowed())return;setPage('commands');setCreateToken(Date.now())}if((e.ctrlKey||e.metaKey)&&e.shiftKey&&e.key.toLowerCase()==='s'){e.preventDefault();if(!navigationAllowed())return;setPage('presets')}};
 window.addEventListener('keydown',onKey);return()=>window.removeEventListener('keydown',onKey);
 },[]);
 useEffect(()=>{if(!desktop)return;let active=true;const off: (()=>void)[]=[];const track=(p:Promise<()=>void>)=>p.then(f=>active?off.push(f):f());
 track(listen<{profileId:string;sink?:string;src:string;volume:number;id:string}>('audio-play',e=>{const p=e.payload;if(p.sink!=='local')return;void playViewerSound({profileId:p.profileId,src:p.src,volume:p.volume,onStart:()=>{void api('audio.ack',{profileId:p.profileId,id:p.id,phase:'started'}).catch(()=>{})}}).then(()=>{void api('audio.ack',{profileId:p.profileId,id:p.id,phase:'done'}).catch(()=>{})}).catch(err=>{notify(errorText(err));void api('audio.ack',{profileId:p.profileId,id:p.id,phase:'error',detail:errorText(err)}).catch(()=>{})})}));
 track(listen('audio-stopLocal',()=>stopViewerSounds()));
 track(listen<Log>('activity',e=>setSnapshot(s=>({...s,logs:[e.payload,...s.logs].slice(0,300)}))));
 track(listen<{profileId:string;status:string}>('connection',e=>setSnapshot(s=>({...s,statuses:{...s.statuses,[e.payload.profileId]:e.payload.status}}))));
 return()=>{active=false;off.forEach(f=>f())};
 },[]);
 const go=(id:string)=>{if(id===page){setSidebar(false);setPalette(false);return}if(!navigationAllowed())return;setRecipeSection(undefined);setPage(id);setSidebar(false);setPalette(false)};
 async function changeTheme(next:string){setTheme(next);try{await api('settings',{key:'theme',value:next})}catch(e){notify(errorText(e))}}
 async function connection(p:Profile){try{await api(snapshot.statuses[p.id]==='online'?'disconnect':'connect',{profileId:p.id});await refresh()}catch(e){notify(errorText(e))}}
 const logs=snapshot.logs.filter(l=>!profileId||l.profileId===profileId);
 const title=page==='settings'?'Configurações':page==='voicecmds'?'Comandos por voz':nav.find(n=>n.id===page)?.label||'BotLive';
 if(locked)return <Login onLogin={()=>{setLocked(false);void refresh()}}/>;
 return <div className="app"><aside className={'sidebar '+(sidebar?'visible':'')}><a className="brand" href="#" onClick={e=>{e.preventDefault();go('dashboard')}}><span className="brand-mark"><Bot size={25}/></span><span>BotLive<span className="brand-dot">.</span></span><small>DESKTOP</small></a><button className="workspace-picker" onClick={()=>go('profiles')}><span className="avatar">{profile?.name.slice(0,1).toUpperCase()||'B'}</span><span><strong>{profile?.name||'Seu espaço'}</strong><small>{profile?platforms[profile.platform]:'Vamos começar'}</small></span><ChevronDown size={15}/></button><nav>{nav.map(n=><div key={n.id}>{n.group&&<div className="nav-group">{n.group}</div>}<button aria-label={n.label} aria-current={page===n.id?'page':undefined} className={'nav-item '+(page===n.id?'active':'')} onClick={()=>go(n.id)}><n.icon size={18}/><span>{n.label}</span>{n.id==='ai'&&<span className="nav-spark">✦</span>}</button></div>)}</nav><div className="sidebar-bottom"><div className="local-badge"><span className="status-dot"/><span>Seu bot. Seus dados.<small>Rodando localmente</small></span><ShieldCheck size={17}/></div><button aria-label='Configurações' className={'nav-item '+(page==='settings'?'active':'')} onClick={()=>go('settings')}><SettingsIcon size={18}/><span>Configurações</span><small>v{version}</small></button></div></aside><div className="main-shell"><header className="topbar"><div className="row"><button className="icon-button menu-toggle" aria-label="Abrir menu" onClick={()=>setSidebar(!sidebar)}><Menu size={20}/></button><span className="breadcrumb">Meu espaço <span>/</span> <strong>{title}</strong></span></div><div className="row">{profile&&!(page==='dashboard'&&dashboardPrivacy)&&<ChatPresence key={profile.id} profileId={profile.id} online={snapshot.statuses[profile.id]==='online'}/>}<button className="quick-search" onClick={()=>setPalette(true)}><Search size={16}/><span>Ir para…</span><kbd>Ctrl K</kbd></button><button className="icon-button" aria-label="Alternar tema" onClick={()=>changeTheme(theme==='dark'?'light':'dark')}>{theme==='dark'?<Sun size={18}/>:<Moon size={18}/>}</button><span className="top-avatar">{profile?.name.slice(0,1).toUpperCase()||'B'}</span></div></header>{!desktop&&<div className="preview-banner">PRÉVIA NO NAVEGADOR <span>Edite configurações aqui. Conexões, IA e automações funcionam no aplicativo desktop.</span></div>}<main><div className="page-heading"><div><div className="eyebrow">{page==='dashboard'?'BEM-VINDO AO SEU ESPAÇO':'BOTLIVE / '+title.toUpperCase()}</div><h1>{page==='dashboard'?'Sua live, bem acompanhada.':title}</h1><p>{subtitles[page]}</p></div>{page==='dashboard'?<div className="row dashboard-heading-actions">{profile&&<select className="profile-select" aria-label="Perfil ativo" value={profileId} onChange={e=>{if(navigationAllowed())setProfileId(e.target.value)}}>{snapshot.profiles.map(p=><option key={p.id} value={p.id}>{p.name}</option>)}</select>}<button className="primary" onClick={()=>{if(!profile)setProfileModal('new');else{go('commands');setCreateToken(Date.now())}}}><Plus size={17}/>{profile?'Criar comando':'Criar primeiro bot'}</button></div>:profile&&page!=='profiles'&&page!=='settings'?<select className="profile-select" aria-label="Perfil ativo" value={profileId} onChange={e=>{if(navigationAllowed())setProfileId(e.target.value)}}>{snapshot.profiles.map(p=><option key={p.id} value={p.id}>{p.name}</option>)}</select>:null}</div>
 {loading?<div className="empty"><Radio className="spin"/><p>Abrindo seu espaço…</p></div>:<ScreenBoundary key={page+'|'+profileId+'|'+screenNonce} onHome={()=>{setScreenNonce(n=>n+1);go('dashboard')}}>
 {page==='dashboard'&&<Dashboard key={profile?.id||'empty'} profile={profile} botStatus={profile?snapshot.statuses[profile.id]||'offline':'offline'} logs={logs} onGo={go} onPrivacy={setDashboardPrivacy} onCreate={()=>setProfileModal('new')}/>}
 {page==='profiles'&&<><div className="section-tools"><p className="help">{snapshot.profiles.length} perfis · memórias e comandos independentes</p><button className="primary" onClick={()=>setProfileModal('new')}><Plus size={16}/>Novo perfil</button></div><div className="profile-grid">{snapshot.profiles.map(p=>{const st=snapshot.statuses[p.id];const online=st==='online';return <Card key={p.id} className={'profile-card '+(online?'is-online':'is-offline')}><div className="profile-top"><span className={'platform-avatar '+p.platform}><Bot size={25}/></span><div className="profile-id"><h2>{p.name}</h2><div className="profile-meta"><p className="help">{platforms[p.platform]} · {p.channel||'Canal não vinculado'}</p><span className={'status-pill '+(online?'on':'off')}><span className="status-dot"/>{statusText(st)}</span></div></div></div><footer className="profile-actions"><button onClick={()=>setProfileModal(p)}>Configurar</button><button className={online?'':'primary'} onClick={()=>connection(p)}><Power size={15}/>{online?'Desconectar':'Conectar'}</button><button className="icon-button danger" aria-label={'Apagar perfil '+p.name} onClick={()=>setDeleteProfile(p)}><Trash2 size={16}/></button></footer></Card>})}</div>{!snapshot.profiles.length&&<Empty title="Dê um nome ao seu próximo parceiro de live" action={<button className="primary" onClick={()=>setProfileModal('new')}>Criar perfil</button>}>Conecte seu canal Twitch, YouTube ou configure uma integração Kick.</Empty>}</>}
 {['commands','flows','timers','responses','counters','ai','memory','community','presets','discord','labels','sounds','audio','obs','alertas','chatobs','spotify'].includes(page)&&!profile&&<Empty title="Primeiro, um espaço para seu bot" action={<button className="primary" onClick={()=>setProfileModal('new')}><Plus size={16}/>Criar perfil</button>}>Crie um perfil para organizar as configurações do seu canal.</Empty>}
 {profile&&['commands','flows'].includes(page)&&<Commands key={page+profile.id} profile={profile} visual={page==='flows'} notify={notify} onChanged={refreshFlows} createToken={createToken} onGoVoice={()=>go('voicecmds')} onNavigate={(id,section)=>{go(id);setRecipeSection(section)}}/>}
 {profile&&page==='timers'&&<Timers key={profile.id} profile={profile} notify={notify} onChanged={refreshFlows}/>}
 {profile&&page==='responses'&&<ChatExtras key={profile.id} profileId={profile.id}/>}
 {profile&&page==='counters'&&<Counters key={profile.id} profile={profile} notify={notify} onGoCommands={()=>go('commands')}/>}
 {profile&&page==='ai'&&<AIScreen key={profile.id} profile={profile} onSaved={refresh} notify={notify}/>}
 {profile&&page==='memory'&&<MemoryScreen key={profile.id} profile={profile} notify={notify}/>}
 {profile&&page==='community'&&<Community key={profile.id} profile={profile} onSaved={refresh} notify={notify} onOpenResponses={()=>go('responses')} initialSection={recipeSection}/>}
 {profile&&page==='discord'&&<Discord key={profile.id} profileId={profile.id} blocklist={profile.blocklist||[]} notify={notify}/>}
  {profile&&page==='labels'&&<Labels key={profile.id} profileId={profile.id} notify={notify}/>}
  {profile&&page==='sounds'&&<Sounds key={profile.id} profileId={profile.id} notify={notify}/>}
  {profile&&page==='obs'&&<Obs key={profile.id} profileId={profile.id} notify={notify}/>}
  {profile&&page==='alertas'&&<Alertas key={profile.id} profileId={profile.id} notify={notify}/>}
  {profile&&page==='chatobs'&&<ChatObs key={profile.id} profileId={profile.id} notify={notify}/>}

  {profile&&page==='spotify'&&<SpotifyScreen key={profile.id} profileId={profile.id} notify={notify}/>}
  {profile&&page==='audio'&&<AudioHub key={profile.id} profileId={profile.id} notify={notify}/>}
 {profile&&page==='vozfx'&&<Voicemod key={profile.id} profileId={profile.id} notify={notify} initialSection={recipeSection}/>}
 {profile&&page==='voicecmds'&&<VoiceCommandsPage key={profile.id} profileId={profile.id} notify={notify} onBack={()=>go('commands')}/>}
 {profile&&page==='livepix'&&<LivePix key={profile.id} profileId={profile.id} notify={notify}/>}
 {profile&&page==='presets'&&<Presets key={profile.id} profile={profile} notify={notify} onChanged={()=>{void refresh();refreshFlows()}}/>}
 {page==='history'&&<ExecutionHistory key={profileId} profileId={profileId} events={<><div className="section-tools"><div className="search"><Search size={17}/><input aria-label="Filtrar histórico" placeholder="Buscar eventos, ações e erros…" value={historyQuery} onChange={e=>setHistoryQuery(e.target.value)}/></div><Tag color="gray">Até 300 registros recentes</Tag></div><Card>{logs.length?<ActivityList logs={logs.filter(l=>(l.message+l.kind+l.status).toLowerCase().includes(historyQuery.toLowerCase()))}/>:<Empty title="Tudo o que o bot fizer aparece aqui">Conecte um canal ou execute uma simulação para começar.</Empty>}</Card></>}/>}
 {page==='stats'&&(profile?<Stats profileId={profile.id} notify={notify}/>:<Empty title="Crie um perfil para acompanhar sua comunidade">As estatísticas usam os eventos recebidos de cada canal.</Empty>)}
 {page==='settings'&&<Settings profileId={profileId} onLock={()=>setLocked(true)} theme={theme} onTheme={changeTheme} notify={notify} dataDir={snapshot.dataDir}/>}
 </ScreenBoundary>}<footer className="main-footer"><span>Feito para quem faz comunidade.</span><span><span className="status-dot"/> LOCAL FIRST <span>·</span> BOTLIVE {version}</span></footer></main></div>
 {profileModal&&<Modal wide title={profileModal==='new'?'Criar perfil de bot':'Configurar perfil'} onClose={()=>setProfileModal(null)}><ProfileForm initial={profileModal==='new'?undefined:profileModal} profiles={snapshot.profiles} onSaved={p=>{setProfileId(p.id);void refresh()}} notify={notify}/></Modal>}
 {deleteProfile&&<Modal title={'Apagar “'+deleteProfile.name+'”?'} onClose={()=>setDeleteProfile(null)}><div className="form-pad"><p>Os comandos, memórias, pontos e credenciais deste perfil serão apagados. Os outros perfis continuarão intactos.</p><footer className="form-footer"><button onClick={()=>setDeleteProfile(null)}>Cancelar</button><button className="danger" onClick={async()=>{try{await api('profile.delete',{profileId:deleteProfile.id});setDeleteProfile(null);await refresh();notify('Perfil apagado.')}catch(e){notify(errorText(e))}}}>Apagar perfil e dados</button></footer></div></Modal>}
 {palette&&<Modal title="Para onde vamos?" onClose={()=>setPalette(false)}><div className="form-pad"><div className="search"><Search size={18}/><input autoFocus aria-label="Buscar tela ou tarefa" placeholder="Busque uma tela ou tarefa: resgate, áudio, erro…" value={search} onChange={e=>setSearch(e.target.value)}/></div><p className="help">Busque pelo nome da tela ou pelo que você quer fazer.</p><div className="palette-results">{[...nav,{id:'settings',label:'Configurações',icon:SettingsIcon}].filter(n=>navigationMatches(n.id,n.label,search)).map(n=><button key={n.id} onClick={()=>go(n.id)}><n.icon size={18}/>{n.label}<ArrowRight size={16}/></button>)}{![...nav,{id:'settings',label:'Configurações'}].some(n=>navigationMatches(n.id,n.label,search))&&<p role="status" className="help">Nenhuma tela encontrada. Tente “comando”, “resgate”, “áudio” ou “backup”.</p>}</div></div></Modal>}
 {toast&&<div className="toast" role="status"><span>{toast}</span><button className="icon-button" aria-label="Fechar aviso" onClick={()=>setToast('')}><X size={16}/></button></div>}
 </div>
}
function statusText(s?:string){return s==='online'?'Conectado':s==='connecting'?'Conectando':s==='reconnecting'?'Reconectando':'Desconectado'}
function ActivityList({logs}:{logs:Log[]}){return <div className="activity-list">{logs.map(l=><div key={l.id} className="activity-row"><span className={'activity-symbol '+(l.status==='error'?'red':l.status==='success'?'green':'blue')}>{l.status==='success'?<Check size={15}/>:<Zap size={15}/>}</span><div><span className="activity-kind">{l.kind}</span><p>{l.message}</p></div><time>{fmtTime((l as {timestamp?:unknown}).timestamp??(l as {at?:unknown}).at)}</time></div>)}</div>}
