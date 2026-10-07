import CommandOptions from './CommandOptions';
import {useState,useCallback,useEffect,useRef,type ReactNode} from 'react';
import {ReactFlow,Background,Controls,MiniMap,addEdge,useNodesState,useEdgesState,useReactFlow,useStore,type Connection,type Node,type Edge} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import {Plus,Save,Trash2,Zap,Play,Map as MapIcon} from 'lucide-react';
import {actions,triggers,newAction,punishModes,punishTargets,twOps,condOps,condFalse,minutesToSeconds,secondsToMinutes,type Flow,type Action,type AIConfig} from './types';
import {orderedActions,moveAction,chainIds,stepBounds,fullChain,appendStepPlan,insertStepPlan,type StepPlan} from './flow';
import {StepMenu,FlowHelp} from './FlowChrome';
import {readPrefs,writePrefs} from './editorPrefs';
import {Field,Toggle} from './components';
import {VariableTarget} from './Variables';
import {stepLabel} from './stepLabel';
import VariablePicker from './VariablePicker';
import MessageEditor from './MessageEditor';
import {SendPicker,AudioPicker} from './FlowOptions';
import {AIResponseTest} from './AIConversation';
import AIActionOptions from './AIActionOptions';
import {friendlyError,friendlyMessage,type FriendlyError} from './errors';
import {api,desktop} from './api';
const OBS_OPS:Record<string,string>={mute:'Mutar entrada',unmute:'Desmutar entrada',toggle_mute:'Alternar mudo',volume:'Volume da entrada',show:'Mostrar fonte',hide:'Esconder fonte',toggle_item:'Alternar fonte',scene:'Trocar de cena'};
const OBS_TEMP=['mute','unmute','volume','show','hide'];
type ObsDisco={scenes:string[];current:string;inputs:string[];sources:string[]};
function ObsTarget({profileId,op,value,onChange,issue}:{profileId:string;op:string;value:string;onChange:(v:string)=>void;issue?:string}){
 const [disco,setDisco]=useState<ObsDisco|null>(null);const [error,setError]=useState('');
 useEffect(()=>{if(!desktop){return}api<ObsDisco>('obs.discover',{profileId}).then(setDisco).catch(()=>setError('Abra o OBS e confira a integração na tela OBS Studio.'))},[profileId]);
 const options=op==='scene'?(disco?.scenes||[]):['mute','unmute','toggle_mute','volume'].includes(op)?(disco?.inputs||[]):(disco?.sources||[]);
 const label=op==='scene'?'Cena':['mute','unmute','toggle_mute','volume'].includes(op)?'Entrada de áudio':'Fonte (cena atual)';
 return <><Field label={label}><select value={value} onChange={e=>onChange(e.target.value)}><option value="">Escolha…</option>{options.map(o=><option key={o} value={o}>{o}</option>)}{value&&!options.includes(value)&&<option value={value}>{value}</option>}</select></Field>{issue&&<p className="field-warn" role="status">{issue}</p>}{error&&<p className="help">{error}</p>}</>;
}
type Data={label:string;action?:Action;[key:string]:unknown};
/** Zoom atual do canvas (o transform do viewport do React Flow), para medir blocos em unidades do fluxo. */
function flowZoom():number{
 const el=document.querySelector('.react-flow__viewport');
 const m=el?(el as HTMLElement).style.transform.match(/scale\(([\d.]+)\)/):null;
 return m?Number(m[1])||1:1;
}
/** Mede os blocos no canvas em unidades do fluxo; usado para abrir espaço sem sobrepor ao adicionar ou inserir. */
function flowSizes():Record<string,{w:number;h:number}>{
 const z=flowZoom();const out:Record<string,{w:number;h:number}>={};
 document.querySelectorAll<HTMLElement>('.react-flow__node').forEach(el=>{
  const id=el.dataset.id;if(!id)return;
  const r=el.getBoundingClientRect();
  out[id]={w:Math.round(r.width/z)||185,h:Math.round(r.height/z)||70};
 });
 return out;
}
/** Deixa a etapa nova visível sem redefinir o zoom: só faz pan quando ela caiu fora do canvas. */
function StepReveal({target,restZoom}:{target:{id:string;x:number;y:number}|null;restZoom:{current:number|null}}){
 const rf=useReactFlow();
 const width=useStore(s=>s.width);
 const height=useStore(s=>s.height);
 const timer=useRef(0);
 useEffect(()=>{
  if(!target||!width||!height)return;
  window.clearTimeout(timer.current);
  timer.current=window.setTimeout(()=>{
   const vp=rf.getViewport();
   // zoom estável da última pausa: uma animação de pan arremessa o zoom para o meio do caminho
   const zoom=restZoom.current??vp.zoom;
   const w=220,h=90;
   const cx=vp.x+(target.x+w/2)*zoom;
   const cy=vp.y+(target.y+h/2)*zoom;
   if(cx>8&&cx<width-8&&cy>8&&cy<height-8)return;
   restZoom.current=zoom;
   rf.setCenter(target.x+w/2,target.y+h/2,{zoom,duration:180});
  },60);
  return ()=>window.clearTimeout(timer.current);
 },[target,width,height,rf,restZoom]);
 return null;
}
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
function CondEditor({action,update,issue}:{action:Action;update:(a:Action)=>void;issue?:string}){
 return <>
  <Field label="Variável"><VariablePicker value={action.condVar||''} onChange={condVar=>update({...action,condVar})}/></Field>
  {issue&&<p className="field-warn" role="status">{issue}</p>}
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
 const startNodes:Node<Data>[]=layout?.nodes?layout.nodes.map(n=>n.data.action?{...n,data:{...n.data,label:stepLabel(n.data.action)}}:n):[{id:'trigger',position:{x:60,y:140},data:{label:'⚡ '+(triggers[flow.trigger.kind]||flow.trigger.kind)},type:'input'},...flow.actions.map((action,i)=>({id:'action-'+i,position:{x:60,y:140+(i+1)*90},data:{label:stepLabel(action),action}}))];
 const startEdges:Edge[]=layout?.edges||flow.actions.map((_,i)=>({id:'e'+i,source:i===0?'trigger':'action-'+(i-1),target:'action-'+i,animated:true}));
 const [nodes,setNodes,onNodesChange]=useNodesState(startNodes);
 const [edges,setEdges,onEdgesChange]=useEdgesState(startEdges);
 const [selected,setSelected]=useState('trigger');const [name,setName]=useState(flow.name);const [trigger,setTrigger]=useState(flow.trigger);const [error,setError]=useState<FriendlyError|null>(null);const [busy,setBusy]=useState(false);
 const node=nodes.find(n=>n.id===selected);const action=node?.data.action;
 const problem=action?actionProblem(action):null;
 const issue=problem&&node?.className==='step-invalid'?problem.message:undefined;
 const [showMini,setShowMini]=useState(()=>!!readPrefs().minimap);
 function toggleMini(){setShowMini(v=>{const next=!v;writePrefs({minimap:next});return next})}
 const stepChain=(()=>{try{return chainIds(nodes,edges).filter(x=>x!=='trigger')}catch{return [] as string[]}})();
 const bounds=stepBounds(stepChain,selected);
 // inserir no meio exige uma única cadeia válida; com bloco solto, ciclo ou ramificação a opção some do menu
 const insertable=(()=>{try{fullChain(nodes,edges);return true}catch{return false}})();
 const [reveal,setReveal]=useState<{id:string;x:number;y:number}|null>(null);
 /** Zoom da última pausa do canvas: animações de pan oscilam o zoom no meio e não servem de alvo. */
 const restZoom=useRef<number|null>(null);
 const connect=useCallback((c:Connection)=>setEdges(e=>addEdge({...c,animated:true},e)),[setEdges]);
 /** Espelha as regras do backend (model.rs) para apontar etapa e campo antes de enviar. */
 function actionProblem(a:Action):{message:string}|null{
  if(a.kind==='condition'&&!(a.condVar||'').trim())return {message:'Escolha uma variável'};
  if(a.kind==='obs'&&!(a.obsTarget||'').trim()){
   const op=a.obsOp||'mute';
   return {message:op==='scene'?'Escolha uma cena':['mute','unmute','toggle_mute','volume'].includes(op)?'Escolha uma entrada de áudio':'Escolha uma fonte'};
  }
  return null;
 }
 function update(a:Action){setNodes(ns=>ns.map(n=>n.id===selected?{...n,data:{label:stepLabel(a),action:a},className:actionProblem(a)?n.className:undefined}:n))}
function move(dir:-1|1){try{const ns=nodes.map(n=>({id:n.id,position:{x:n.position.x,y:n.position.y},data:{action:n.data.action,label:String(n.data.label||'')}}));const es=edges.map(e=>({source:e.source,target:e.target}));const r=moveAction(ns,es,selected,dir);setNodes(cur=>{const pos:Record<string,{x:number;y:number}>={};r.nodes.forEach(n=>{pos[n.id]=n.position});return cur.map(n=>pos[n.id]?{...n,position:pos[n.id]}:n)});setEdges(r.edges.map((e,i)=>({id:'e'+i,source:e.source,target:e.target,animated:true})))}catch(e){setError(friendlyError(e))}}
 /** Aplica um plano de cadeia: posiciona os blocos, refaz as conexões, seleciona a etapa nova e pede para deixá-la visível. */
 function applyStep(plan:StepPlan,id:string){
  setNodes(ns=>[...ns.map(n=>plan.positions[n.id]?{...n,position:plan.positions[n.id],selected:false}:{...n,selected:false}),{id,position:plan.positions[id],data:{label:actions.chat,action:newAction()},selected:true}]);
  setEdges(plan.edges.map((e,i)=>({id:'e'+i,source:e.source,target:e.target,animated:true})));
  setSelected(id);
  setReveal({id,x:plan.positions[id].x,y:plan.positions[id].y});
 }
 function addStep(){const id=crypto.randomUUID();try{applyStep(appendStepPlan(nodes,edges,id,flowSizes()),id)}catch(e){setError(friendlyError(e))}}
 function insertStep(){const id=crypto.randomUUID();try{applyStep(insertStepPlan(nodes,edges,selected,id,flowSizes()),id)}catch(e){setError(friendlyError(e))}}
 const keepText=(kind:string)=>["command","contains","voice","mention","redemption"].includes(kind);
function switchKind(current:Action,next:string){const base:Action={...current,kind:next};
  if(next==='ai.generate'&&!base.target.startsWith('local.'))base.target='local.aiResponse';
  if(next==='punish'){if(!['sender','first','random'].includes(base.target))base.target='sender';if(!base.punish)base.punish='timeout';if(base.value<1)base.value=60}
  if(next==='condition'){if(!base.condOp)base.condOp='equals';if(!['stop','skip'].includes(base.condFalse||''))base.condFalse='stop'}
  if(next==='twitch'){if(!base.twOp)base.twOp='game';if(base.value<1)base.value=60}
  if(next==='wait'&&!(base.value>=1))base.value=3000
  if(next==='obs'){if(!base.obsOp)base.obsOp='mute';if(!base.obsTarget)base.obsTarget=''}
  update(base)}
 async function save(){
  const invalid=nodes.find(n=>n.data.action&&actionProblem(n.data.action));
  if(invalid){setNodes(ns=>ns.map(n=>n.id===invalid.id?{...n,className:'step-invalid'}:n));setSelected(invalid.id);return}
  setNodes(ns=>ns.map(n=>n.className==='step-invalid'?{...n,className:undefined}:n));
  try{setBusy(true);setError(null);const ordered=orderedActions(nodes,edges);await onSave({...flow,name,trigger:trigger.kind==='timer'?{...trigger,permission:'everyone',cooldown:0,userCooldown:0,pattern:''}:keepText(trigger.kind)?trigger:{...trigger,pattern:''},counter:trigger.kind==='command'&&counter,timerSeconds,sendType,sendColor,replyTo,audio,audioVolume,actions:ordered,layout:{nodes,edges}})}catch(e){setError(friendlyError(e))}finally{setBusy(false)}}
 return <div className="flow-editor">
 <div className="flow-toolbar"><input aria-label="Nome do fluxo" value={name} onChange={e=>setName(e.target.value)}/><FlowHelp note={intro}/>{onTest&&<button type="button" data-test-flow disabled={busy} onClick={()=>{try{onTest({...flow,name,trigger,counter:trigger.kind==='command'&&counter,timerSeconds,sendType,sendColor,replyTo,audio,audioVolume,actions:orderedActions(nodes,edges),layout:{nodes,edges}})}catch(e){setError(friendlyError(e))}}}><Play size={16}/>Testar fluxo</button>}<button type="button" aria-expanded={showMini} onClick={toggleMini}><MapIcon size={16}/>Minimapa</button><button onClick={addStep}><Plus size={16}/>Adicionar etapa</button><button className="primary" disabled={busy} onClick={save}><Save size={16}/>Salvar fluxo</button></div>
 {error&&<div role="alert" className="inline-error">{error.message}{error.detail&&<details><summary>Detalhes técnicos</summary><code>{error.detail}</code></details>}</div>}
 <div className="flow-workspace">
 <div className="canvas">
 <ReactFlow nodes={nodes} edges={edges} onNodesChange={onNodesChange} onEdgesChange={onEdgesChange} onConnect={connect} onNodeClick={(_,n)=>setSelected(n.id)} onMoveEnd={(_,vp)=>{restZoom.current=vp.zoom}} fitView minZoom={0.25} maxZoom={1.5} deleteKeyCode={['Backspace','Delete']}>
 <Background gap={22}/>
 <Controls/>
 {showMini&&<MiniMap pannable zoomable style={{width:150,height:100}}/>}
 <StepReveal target={reveal} restZoom={restZoom}/>
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
 <Section title="Configurar etapa" open>
 <Field label="Tipo de etapa">
 <select value={action.kind} onChange={e=>switchKind(action,e.target.value)}>{Object.entries(actions).map(([k,n])=>
 <option key={k} value={k}>{n}</option>)}</select>
 </Field>
 <div className="list-row"><div><strong>Etapa ativa</strong><small>Desligada pula sem erro.</small></div><Toggle label="Etapa ativa" checked={action.enabled!==false} onChange={v=>update({...action,enabled:v})}/></div>
 {action.kind==='wait'&&<WaitEditor value={action.value||0} onChange={value=>update({...action,value})}/>}
 <div className="row"><StepMenu canUp={bounds.up} canDown={bounds.down} canInsert={insertable} onMove={move} onInsert={insertStep}/></div>
 {action.kind==='condition'&&<CondEditor action={action} update={update} issue={issue}/>}
 {action.kind==='script'?<Field label="Script Rhai (retorna texto)">
 <textarea rows={5} value={action.text} onChange={e=>update({...action,text:e.target.value})}/>
 </Field>:action.kind==='variable.delete'?<p className="help">Escolha o valor que será apagado.</p>:action.kind==='twitch'&&!['game','title','timeout','ban','warn','mention'].includes(action.twOp||'game')?<p className="help">Esta operação não usa texto: o alvo e a duração vêm da fala ou dos campos abaixo.</p>:<MessageEditor key={selected} profileId={flow.profileId} label={action.kind.startsWith('ai')?'Como a IA deve responder':action.kind==='variable.increment'?'Quanto somar':action.kind==='punish'?'Motivo (vai para a Twitch e para o Histórico)':action.kind==='twitch'?((action.twOp||'game')==='game'?'Nome do jogo fixo (vazio usa o que você falou)':(action.twOp||'game')==='title'?'Novo título fixo (vazio usa o que você falou, até 140 caracteres)':(action.twOp||'game')==='mention'?'Mensagem (o bot marca @alvo na frente)':'Motivo (vai para a Twitch e para o Histórico)'):'Mensagem / conteúdo'} value={action.text} onChange={text=>update({...action,text})} flow={{...flow,name,trigger,actions:(()=>{try{return orderedActions(nodes,edges)}catch{return []}})()}}/>}{ai&&<AIActionOptions action={action} ai={ai} onChange={update}/>}{action.kind==='ai.generate'&&<Field label="Nome da resposta" hint="Disponível somente nesta execução; evita misturar respostas entre pessoas.">
 <input value={(action.target||'local.aiResponse').replace(/^local\./,'')} onChange={e=>update({...action,target:'local.'+e.target.value})}/>
 </Field>}{action.kind.startsWith('variable.')&&<VariableTarget value={action.target} onChange={target=>update({...action,target})}/>}{['memory','webhook'].includes(action.kind)&&<Field label={action.kind==='memory'?'Arquivo no vault':'Endereço HTTPS'}>
 <input value={action.target} onChange={e=>update({...action,target:e.target.value})}/>
 </Field>}{['delay','points'].includes(action.kind)&&<Field label={action.kind==='delay'?'Espera em milissegundos (máx. 30000)':'Quantidade de pontos'}>
 <input type="number" value={action.value} onChange={e=>update({...action,value:+e.target.value})}/>
 </Field>}{action.kind==='punish'&&<>
 <Field label="O que aplicar">
 <select value={action.punish||'timeout'} onChange={e=>update({...action,punish:e.target.value})}>{Object.entries(punishModes).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select>
 </Field>
 {(action.punish||'timeout')==='timeout'&&<Field label="Duração do silêncio (segundos)" hint="De 1 segundo a 14 dias.">
 <input type="number" min="1" max="1209600" value={action.value} onChange={e=>update({...action,value:+e.target.value})}/>
 </Field>}
 <Field label="Quem leva a punição">
 <select value={action.target} onChange={e=>update({...action,target:e.target.value})}>{Object.entries(punishTargets).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select>
 </Field>
 <p className="help">{action.target==='first'?'Escreva o alvo depois do comando, como !silenciar @alvo. O nome é convertido em ID na Twitch antes da ação.':'A ação pune quem disparou o gatilho. Em comando de todo mundo, restrinja em Quem pode usar para Moderadores ou Só o streamer.'}</p>
 <p className="help">Só executa em perfil Twitch com a conta do canal autorizada. A prévia não pune ninguém e o Histórico registra o resultado.</p>
 </>}{action.kind==='twitch'&&<>
 <Field label="Operação">
 <select value={action.twOp||'game'} onChange={e=>update({...action,twOp:e.target.value})}>{Object.entries(twOps).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select>
 </Field>
 {['timeout','ban','unban','warn','vip','unvip','shoutout','mention'].includes(action.twOp||'game')&&<Field label="Alvo fixo (opcional)" hint="Vale quando a fala não traz @menção nem nome de quem está no chat. Ex.: maria.">
 <input value={action.target} onChange={e=>update({...action,target:e.target.value})}/>
 </Field>}
 {(action.twOp||'game')==='timeout'&&<Field label="Duração em segundos" hint="Vale o primeiro número da fala; senão, este valor. De 1 segundo a 14 dias. Fale os dígitos, como 300.">
 <input type="number" min="1" max="1209600" value={action.value} onChange={e=>update({...action,value:+e.target.value})}/>
 </Field>}
 {(action.twOp||'game')==='slow'&&<Field label="Intervalo em segundos" hint="Vale o primeiro número da fala; senão, este valor. De 3 a 120.">
 <input type="number" min="3" max="120" value={action.value} onChange={e=>update({...action,value:+e.target.value})}/>
 </Field>}
 {(action.twOp||'game')==='followers'&&<Field label="Só seguidores há (minutos)" hint="Vale o primeiro número da fala; senão, este valor. 0 exige só o follow, sem tempo mínimo.">
 <input type="number" min="0" max="129600" value={action.value} onChange={e=>update({...action,value:+e.target.value})}/>
 </Field>}
 <p className="help">O alvo sai da fala: @menção primeiro, depois nome de quem está no chat, depois o alvo fixo. Sem nenhum, o bot avisa no Histórico e não executa.</p>
 <p className="help">Categoria, título e VIP executam com a conta do canal (reautorize o canal após atualizar); o resto executa com a conta do bot, que precisa ser moderadora. A prévia não executa.</p>
 </>}{action.kind==='obs'&&<>
  <Field label="Operação no OBS"><select value={action.obsOp||'mute'} onChange={e=>update({...action,obsOp:e.target.value,obsTarget:''})}>{Object.entries(OBS_OPS).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>
  <ObsTarget profileId={flow.profileId} op={action.obsOp||'mute'} value={action.obsTarget||''} onChange={obsTarget=>update({...action,obsTarget})} issue={issue}/>
  {(action.obsOp||'mute')==='volume'&&<Field label="Volume (dB)" hint="Ex.: -6. Entre -100 e 30."><input type="number" min={-100} max={30} step="any" value={action.text} onChange={e=>update({...action,text:e.target.value})}/></Field>}
  {OBS_TEMP.includes(action.obsOp||'mute')&&<Field label="Duração (segundos)" hint="0 = permanente. Temporário restaura o estado anterior no fim."><input type="number" min={0} max={3600} step={1} value={action.obsDuration??0} onChange={e=>update({...action,obsDuration:Math.max(0,Math.round(Number(e.target.value)||0))})}/></Field>}
  <p className="help">Executa no OBS desta máquina, sem nuvem. Temporário com o mesmo alvo soma o tempo; alvo ocupado por outro efeito é recusado no Histórico.</p>
  </>}</Section>
  {(action.kind.startsWith('ai')||action.kind==='obs')&&<Section title="Avançado">
  {action.kind.startsWith('ai')&&<details className="ai-help"><summary>Como a IA monta a resposta</summary><p className="help">A mensagem atual, até 12 falas recentes dos últimos 5 minutos e as memórias entram automaticamente. Defina o tom aqui. {action.kind==='ai.generate'?'Esta ação só guarda a resposta. Conecte Enviar mensagem e clique em + Resposta da IA.':'Esta ação já envia ao chat e guarda a resposta para os próximos blocos.'}</p></details>}
  {action.kind.startsWith('ai')&&<AIResponseTest key={selected} profileId={flow.profileId} instruction={action.text}/>}
  {action.kind==='obs'&&<><div className="row"><button type="button" disabled={busy||!desktop} onClick={async()=>{setBusy(true);setObsTest('');try{const done=await api<string>('obs.execute',{profileId:flow.profileId,op:action.obsOp||'mute',target:action.obsTarget||'',num:action.obsOp==='volume'?Number(action.text)||0:0,secs:action.obsDuration??0});setObsTest(String(done))}catch(e){setObsTest('Falhou: '+friendlyMessage(e))}finally{setBusy(false)}}}><Play size={15}/>Testar no OBS</button></div>{obsTest&&<p className="help">{obsTest}</p>}</>}
  </Section>}
 <Section title="Comportamento">
 <Field label="Executar só se a mensagem contiver">
 <input value={action.condition} onChange={e=>update({...action,condition:e.target.value})}/>
 </Field>
 <p className="help">A ação é pulada quando o texto não contém o trecho, sem diferenciar maiúsculas de minúsculas. As ações seguintes continuam normalmente.</p>
 </Section>
 <button className="danger" onClick={()=>{setNodes(ns=>ns.filter(n=>n.id!==selected));setEdges(es=>es.filter(e=>e.source!==selected&&e.target!==selected));setSelected('trigger')}}>
 <Trash2 size={15}/>Remover bloco</button>
 </>:<p>Selecione um bloco para editar.</p>}<p className="help">A ordem é a das setas entre os blocos, não a posição deles na tela: arraste de um ponto de conexão ao outro e mantenha uma única sequência começando no gatilho. Blocos desconectados ficam de fora e o salvamento avisa.</p>
 </aside>
 </div>
 </div>
}
