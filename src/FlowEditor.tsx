import CommandOptions from './CommandOptions';
import {useState,useCallback,useEffect,type ReactNode} from 'react';
import {ReactFlow,Background,Controls,MiniMap,addEdge,useNodesState,useEdgesState,type Connection,type Node,type Edge} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import {Plus,Save,Trash2,Zap,Play} from 'lucide-react';
import {actions,triggers,newAction,punishModes,punishTargets,twOps,condOps,condFalse,minutesToSeconds,secondsToMinutes,type Flow,type Action,type AIConfig} from './types';
import {orderedActions,moveAction} from './flow';
import {Field,Toggle} from './components';
import {VariableTarget} from './Variables';
import {variableCatalog} from './variableCatalog';
import MessageEditor from './MessageEditor';
import {SendPicker,AudioPicker} from './FlowOptions';
import {AIResponseTest} from './AIConversation';
import AIActionOptions from './AIActionOptions';
import {friendlyError,friendlyMessage,type FriendlyError} from './errors';
import {api,desktop} from './api';
const OBS_OPS:Record<string,string>={mute:'Mutar entrada',unmute:'Desmutar entrada',toggle_mute:'Alternar mudo',volume:'Volume da entrada',show:'Mostrar fonte',hide:'Esconder fonte',toggle_item:'Alternar fonte',scene:'Trocar de cena'};
const OBS_TEMP=['mute','unmute','volume','show','hide'];
type ObsDisco={scenes:string[];current:string;inputs:string[];sources:string[]};
function ObsTarget({profileId,op,value,onChange}:{profileId:string;op:string;value:string;onChange:(v:string)=>void}){
 const [disco,setDisco]=useState<ObsDisco|null>(null);const [error,setError]=useState('');
 useEffect(()=>{if(!desktop){return}api<ObsDisco>('obs.discover',{profileId}).then(setDisco).catch(()=>setError('Abra o OBS e confira a integração na tela OBS Studio.'))},[profileId]);
 const options=op==='scene'?(disco?.scenes||[]):['mute','unmute','toggle_mute','volume'].includes(op)?(disco?.inputs||[]):(disco?.sources||[]);
 const label=op==='scene'?'Cena':['mute','unmute','toggle_mute','volume'].includes(op)?'Entrada de áudio':'Fonte (cena atual)';
 return <><Field label={label}><select value={value} onChange={e=>onChange(e.target.value)}><option value="">Escolha…</option>{options.map(o=><option key={o} value={o}>{o}</option>)}{value&&!options.includes(value)&&<option value={value}>{value}</option>}</select></Field>{error&&<p className="help">{error}</p>}</>;
}
type Data={label:string;action?:Action;[key:string]:unknown};
/** Seção recolhível do painel lateral: agrupa campos parecidos e sai do caminho quando fechada. */
type Reward={id:string;title:string;cost:number;enabled:boolean;paused:boolean};
function RewardPicker({profileId,value,onChange}:{profileId:string;value:string;onChange:(id:string)=>void}){
 const [rewards,setRewards]=useState<Reward[]>([]);const [error,setError]=useState('');
 useEffect(()=>{if(!desktop)return;api<Reward[]>('twitch.rewards',{profileId}).then(setRewards).catch(()=>setError('Não foi possível listar as recompensas. Confira a conexão e a autorização do canal.'))},[profileId]);
 if(!desktop)return <p className="help">A lista de recompensas aparece no aplicativo desktop.</p>;
 const found=rewards.find(r=>r.id===value);
 return <><Field label="Recompensa" hint="Identidade por ID: renomear na Twitch não quebra. Vazio vale qualquer resgate."><select value={value} onChange={e=>onChange(e.target.value)}><option value="">Qualquer resgate</option>{rewards.map(r=><option key={r.id} value={r.id}>{r.title} · {r.cost} pts{r.enabled===false?' (desativada)':''}</option>)}{value&&!found&&<option value={value}>{value}</option>}</select></Field>{error&&<p className="help">{error}</p>}</>;
}
function Section({title,open=false,children}:{title:string;open?:boolean;children:ReactNode}){return <details open={open}><summary>{title}</summary>{children}</details>}
function WaitEditor({value,onChange}:{value:number;onChange:(ms:number)=>void}){
 const units=[['ms',1,'milissegundos'],['s',1000,'segundos'],['min',60000,'minutos']] as const;
 const initial=(units.find(([,f])=>value%f===0&&value/f>=1)||units[0]);
 const [unit,setUnit]=useState<string>(initial[0]);
 const factor=units.find(([u])=>u===unit)?.[1]||1;
 const shown=Math.round((value/factor)*100)/100;
 return <div className="row end"><Field label="Duração"><input type="number" min={0} step="any" value={shown} onChange={e=>onChange(Math.max(0,Math.round((Number(e.target.value)||0)*factor)))}/></Field><Field label="Unidade"><select value={unit} onChange={e=>{const f=units.find(([u])=>u===e.target.value)?.[1]||1;setUnit(e.target.value);onChange(Math.max(0,Math.round((value/f)*f)))}}>{units.map(([u,,n])=><option key={u} value={u}>{n}</option>)}</select></Field></div>;
}
function CondEditor({action,update}:{action:Action;update:(a:Action)=>void}){
 const groups:Record<string,{key:string;label:string}[]>={};
 for(const v of variableCatalog){(groups[v.group]=groups[v.group]||[]).push({key:v.key,label:v.label})}
 return <>
  <Field label="Variável"><select value={action.condVar||''} onChange={e=>update({...action,condVar:e.target.value})}>
   <option value="">Escolha…</option>
   {Object.entries(groups).map(([g,vs])=><optgroup key={g} label={g}>{vs.map(v=><option key={v.key} value={v.key}>{v.label}</option>)}</optgroup>)}
  </select></Field>
  <Field label="Operador"><select value={action.condOp||'equals'} onChange={e=>update({...action,condOp:e.target.value})}>{Object.entries(condOps).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>
  {!['is_empty','is_not_empty'].includes(action.condOp||'equals')&&<Field label="Valor esperado"><input value={action.condValue||''} maxLength={500} onChange={e=>update({...action,condValue:e.target.value})}/></Field>}
  <Field label="Se falso"><select value={action.condFalse||'stop'} onChange={e=>update({...action,condFalse:e.target.value})}>{Object.entries(condFalse).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>
 </>;
}
export default function FlowEditor({flow,platform='twitch',ai,onSave,onTest,intro}:{flow:Flow;platform?:string;ai?:AIConfig;onSave:(f:Flow)=>Promise<void>;onTest?:(f:Flow)=>void;intro?:string}){
 const [counter,setCounter]=useState(!!flow.counter);const [timerSeconds,setTimerSeconds]=useState(flow.timerSeconds||300);
 const [sendType,setSendType]=useState(flow.sendType||'chat');const [sendColor,setSendColor]=useState(flow.sendColor||'primary');const [replyTo,setReplyTo]=useState(!!flow.replyTo);
 const [audio,setAudio]=useState(flow.audio||'');const [audioVolume,setAudioVolume]=useState(flow.audioVolume??1);const [obsTest,setObsTest]=useState('');
 const layout=flow.layout as {nodes?:Node<Data>[];edges?:Edge[]}|undefined;
 const startNodes:Node<Data>[]=layout?.nodes||[{id:'trigger',position:{x:60,y:140},data:{label:'⚡ '+(triggers[flow.trigger.kind]||flow.trigger.kind)},type:'input'},...flow.actions.map((action,i)=>({id:'action-'+i,position:{x:60,y:140+(i+1)*90},data:{label:nodeLabel(action),action}}))];
 const startEdges:Edge[]=layout?.edges||flow.actions.map((_,i)=>({id:'e'+i,source:i===0?'trigger':'action-'+(i-1),target:'action-'+i,animated:true}));
 const [nodes,setNodes,onNodesChange]=useNodesState(startNodes);
 const [edges,setEdges,onEdgesChange]=useEdgesState(startEdges);
 const [selected,setSelected]=useState('trigger');const [name,setName]=useState(flow.name);const [trigger,setTrigger]=useState(flow.trigger);const [error,setError]=useState<FriendlyError|null>(null);const [busy,setBusy]=useState(false);
 const node=nodes.find(n=>n.id===selected);const action=node?.data.action;
 const connect=useCallback((c:Connection)=>setEdges(e=>addEdge({...c,animated:true},e)),[setEdges]);
 function update(a:Action){setNodes(ns=>ns.map(n=>n.id===selected?{...n,data:{label:nodeLabel(a),action:a}}:n))}
function move(dir:-1|1){try{const ns=nodes.map(n=>({id:n.id,position:{x:n.position.x,y:n.position.y},data:{action:n.data.action,label:String(n.data.label||'')}}));const es=edges.map(e=>({source:e.source,target:e.target}));const r=moveAction(ns,es,selected,dir);setNodes(cur=>{const pos:Record<string,{x:number;y:number}>={};r.nodes.forEach(n=>{pos[n.id]=n.position});return cur.map(n=>pos[n.id]?{...n,position:pos[n.id]}:n)});setEdges(r.edges.map((e,i)=>({id:'e'+i,source:e.source,target:e.target,animated:true})))}catch(e){setError(friendlyError(e))}}
 const keepText=(kind:string)=>["command","contains","voice","mention","redemption"].includes(kind);
  function nodeLabel(a:Action):string{const kind=a.kind;
  if(kind==='condition')return '◆ '+(condVarLabel(a.condVar||''))+' '+condOpLabel(a.condOp||'equals')+(a.condValue?' '+a.condValue:'');
  if(kind==='wait')return '◷ '+durationLabel(a.value||0);
  if(kind==='obs')return '◉ '+obsOpLabel(a.obsOp||'mute')+(a.obsTarget?' · '+a.obsTarget:'');
  if(kind==='twitch')return '⚡ '+twitchKindLabel(a.twOp||'game');
  if(kind==='punish')return '🔨 '+(punishModes[a.punish||'timeout']||a.punish||'Punir');
  if(kind==='redemption')return '⚡ Resgate';
  if(kind==='script')return '📜 Script';
  return actions[kind]||kind}
  function condOpLabel(op:string){const m:Record<string,string>={equals:'=',greater_or_equal:'≥',greater_than:'>',less_than:'≤',less_or_equal:'≤',not_equal:'≠',is_empty:'vazio',is_not_empty:'não vazio'};return m[op]||op}
  function condFalseLabel(f:string){const m:Record<string,string>={stop:'PARAR FLUXO',skip:'PULAR PRÓXIMO'};return m[f]||f}
  function durationLabel(ms:number){const v=ms/1000;if(v>=60)return `${Math.round(v/60)} min`;return `${v.toFixed(1)} s`}
  function twitchKindLabel(k:string){const m:Record<string,string>={game:'Jogo',title:'Titulo',timeout:'Timeout',ban:'Ban',unban:'Desban',warn:'Warn',vip:'Vip',unvip:'Unvip',shoutout:'Shoutout',mention:'Mencao',followers:'Followers'};return m[k]||k}
   function condVarLabel(k:string){const v=variableCatalog.find(v=>v.key===k);return v?v.label:k}
   function obsOpLabel(op:string){const m:Record<string,string>={mute:'Mutar',unmute:'Desmutar',toggle_mute:'Alternar mudo',volume:'Volume',show:'Mostrar fonte',hide:'Esconder fonte',toggle_item:'Alternar fonte',scene:'Trocar cena'};return m[op]||op}
function switchKind(current:Action,next:string){const base:Action={...current,kind:next};
  if(next==='ai.generate'&&!base.target.startsWith('local.'))base.target='local.aiResponse';
  if(next==='punish'){if(!['sender','first','random'].includes(base.target))base.target='sender';if(!base.punish)base.punish='timeout';if(base.value<1)base.value=60}
  if(next==='condition'){if(!base.condOp)base.condOp='equals';if(!['stop','skip'].includes(base.condFalse||''))base.condFalse='stop'}
  if(next==='twitch'){if(!base.twOp)base.twOp='game';if(base.value<1)base.value=60}
  if(next==='wait'&&!(base.value>=1))base.value=3000
  if(next==='obs'){if(!base.obsOp)base.obsOp='mute';if(!base.obsTarget)base.obsTarget=''}
  update(base)}
 async function save(){try{setBusy(true);setError(null);const ordered=orderedActions(nodes,edges);await onSave({...flow,name,trigger:trigger.kind==='timer'?{...trigger,permission:'everyone',cooldown:0,userCooldown:0,pattern:''}:keepText(trigger.kind)?trigger:{...trigger,pattern:''},counter:trigger.kind==='command'&&counter,timerSeconds,sendType,sendColor,replyTo,audio,audioVolume,actions:ordered,layout:{nodes,edges}})}catch(e){setError(friendlyError(e))}finally{setBusy(false)}}
 return <div className="flow-editor">
 {intro&&<p className="notice flow-intro">{intro}</p>}
 <div className="flow-toolbar"><input aria-label="Nome do fluxo" value={name} onChange={e=>setName(e.target.value)}/>{onTest&&<button type="button" disabled={busy} onClick={()=>{try{onTest({...flow,name,trigger,counter:trigger.kind==='command'&&counter,timerSeconds,sendType,sendColor,replyTo,audio,audioVolume,actions:orderedActions(nodes,edges),layout:{nodes,edges}})}catch(e){setError(friendlyError(e))}}}><Play size={16}/>Testar fluxo</button>}<button onClick={()=>{const id=crypto.randomUUID();setNodes(ns=>ns.concat({id,position:{x:30+ns.length*70,y:260},data:{label:nodeLabel(newAction()),action:newAction()}}));setSelected(id)}}><Plus size={16}/>Adicionar etapa</button><button className="primary" disabled={busy} onClick={save}><Save size={16}/>Salvar fluxo</button></div>
 {error&&<div role="alert" className="inline-error">{error.message}{error.detail&&<details><summary>Detalhes técnicos</summary><code>{error.detail}</code></details>}</div>}
 <div className="flow-workspace">
 <div className="canvas">
 <ReactFlow nodes={nodes} edges={edges} onNodesChange={onNodesChange} onEdgesChange={onEdgesChange} onConnect={connect} onNodeClick={(_,n)=>setSelected(n.id)} fitView minZoom={0.25} maxZoom={1.5} deleteKeyCode={['Backspace','Delete']}>
 <Background gap={22}/>
 <Controls/>
 <MiniMap pannable zoomable/>
 </ReactFlow>
 </div>
 <aside className="node-inspector">
 <div className="eyebrow"><Zap size={14}/> CONFIGURAR BLOCO</div>{selected==='trigger'?<>
 <Section title="Quando" open>
 <Field label="Evento">
 <select value={trigger.kind} onChange={e=>{const kind=e.target.value;setTrigger({...trigger,kind,pattern:keepText(kind)?trigger.pattern:''});setNodes(ns=>ns.map(n=>n.id==='trigger'?{...n,data:{...n.data,label:'⚡ '+(triggers[kind]||kind)}}:n))}}>{Object.entries(triggers).filter(([k])=>flow.trigger.kind==='timer'?k==='timer':k!=='timer').map(([k,n])=>
 <option key={k} value={k}>{n}</option>)}</select>
 </Field>{['command','contains','voice','mention'].includes(trigger.kind)&&<Field label="Texto que dispara" hint={trigger.kind==='mention'?'Separe os nomes do bot com vírgula: o gatilho passa quando um deles aparece na mensagem, como palavra inteira. Ex.: Arroba, ArrobaSrv, arromba.':trigger.kind==='contains'?'Separe várias palavras ou frases com vírgula: o gatilho passa quando uma delas aparece na mensagem.':trigger.kind==='command'?'Separe variações com vírgula quando o povo erra o comando: !whislist, !whishlist, !wishlist. Qualquer uma delas dispara.':trigger.kind==='voice'?'Separe variações com vírgula: o gatilho passa quando uma delas aparece na fala. Ex.: troca o jogo, muda o jogo, minecraft.':undefined}>
 <input value={trigger.pattern} onChange={e=>setTrigger({...trigger,pattern:e.target.value})}/>
 </Field>}{trigger.kind==='redemption'&&<RewardPicker profileId={flow.profileId} value={trigger.pattern} onChange={pattern=>setTrigger({...trigger,pattern})}/>}{trigger.kind!=='timer'&&<>
 <Field label="Quem pode usar">
 <select value={trigger.permission} onChange={e=>setTrigger({...trigger,permission:e.target.value})}>
 <option value="everyone">Todo mundo</option>
 <option value="subscriber">Assinantes</option>
 <option value="moderator">Moderadores</option>
 <option value="broadcaster">Só o streamer</option>
 </select>
 </Field>
 <Field label="Intervalo entre usos (minutos)">
 <input type="number" min="0" max="1440" step="any" value={secondsToMinutes(trigger.cooldown)} onChange={e=>setTrigger({...trigger,cooldown:minutesToSeconds(Number(e.target.value)||0)})}/>
 </Field>
 <Field label="Intervalo por pessoa (minutos)">
 <input type="number" min="0" max="1440" step="any" value={secondsToMinutes(trigger.userCooldown)} onChange={e=>setTrigger({...trigger,userCooldown:minutesToSeconds(Number(e.target.value)||0)})}/>
 </Field>
 </>}</Section>
 <Section title="Como sai" open>
 <SendPicker platform={platform} value={sendType} color={sendColor} reply={replyTo} onChange={setSendType} onColor={setSendColor} onReply={setReplyTo}/>
 <AudioPicker profileId={flow.profileId} value={audio} volume={audioVolume} onChange={setAudio} onVolume={setAudioVolume}/>
 </Section>{['command','timer'].includes(trigger.kind)&&<Section title="Comportamento">
 <CommandOptions timer={trigger.kind==='timer'} seconds={timerSeconds} counter={counter} onSeconds={setTimerSeconds} onCounter={setCounter}/>
 </Section>}</>:action?<>
 <Section title="Configuração" open>
 {action.kind==='condition'&&<div className="inspector-row"><span className="ins-type">◆ CONDIÇÃO</span><span className="ins-summary">{condVarLabel(action.condVar||'')} {condOpLabel(action.condOp||'equals')}{action.condValue?' '+action.condValue:''}</span><span className="ins-sub">Se falso → {condFalseLabel(action.condFalse||'stop')}</span></div>}
 {action.kind==='wait'&&<div className="ins-row"><label>Duração</label><input type="number" min={0} step="any" value={(action.value||0)} onChange={e=>update({...action,value:Math.max(0,Math.round(Number(e.target.value)||0))})}/><span className="ins-unit">ms</span></div>}
 {action.kind==='obs'&&<div className="inspector-row"><span className="ins-type">◉ OBS · {obsOpLabel(action.obsOp||'mute')}</span><span className="ins-summary">{action.obsTarget||'Escolha o alvo'}</span></div>}
 {action.kind==='twitch'&&<div className="inspector-row"><span className="ins-type">⚡ Twitch · {twitchKindLabel(action.twOp||'game')}</span></div>}
 {!['condition','wait','obs','twitch'].includes(action.kind)&&<div className="inspector-row"><span className="ins-type">{actions[action.kind]||action.kind}</span></div>}
 <div className="ins-row"><label>Etapa ativa</label><Toggle label="Etapa ativa" checked={action.enabled!==false} onChange={v=>update({...action,enabled:v})}/></div>
 <div className="row"><button type="button" onClick={()=>move(-1)}>↑ Subir</button><button type="button" onClick={()=>move(1)}>↓ Descer</button></div>
 <button className="danger" onClick={()=>{setNodes(ns=>ns.filter(n=>n.id!==selected));setEdges(es=>es.filter(e=>e.source!==selected&&e.target!==selected));setSelected('trigger')}}><Trash2 size={15}/>Remover etapa</button>
 </Section>
</>:<p>Selecione um bloco para editar.</p>}<p className="help">A ordem é a das setas entre os blocos, não a posição deles na tela: arraste de um ponto de conexão ao outro e mantenha uma única sequência começando no gatilho. Blocos desconectados ficam de fora e o salvamento avisa.</p>
 </aside>
 </div>
 </div>
}
