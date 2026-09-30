import CommandOptions from './CommandOptions';
import {useState,useCallback,type ReactNode} from 'react';
import {ReactFlow,Background,Controls,MiniMap,addEdge,useNodesState,useEdgesState,type Connection,type Node,type Edge} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import {Plus,Save,Trash2,Zap} from 'lucide-react';
import {actions,triggers,newAction,punishModes,punishTargets,twOps,minutesToSeconds,secondsToMinutes,type Flow,type Action,type AIConfig} from './types';
import {orderedActions} from './flow';
import {Field} from './components';
import {VariableTarget} from './Variables';
import MessageEditor from './MessageEditor';
import {SendPicker,AudioPicker} from './FlowOptions';
import {AIResponseTest} from './AIConversation';
import AIActionOptions from './AIActionOptions';
import {friendlyError,type FriendlyError} from './errors';
type Data={label:string;action?:Action;[key:string]:unknown};
/** Seção recolhível do painel lateral: agrupa campos parecidos e sai do caminho quando fechada. */
function Section({title,open=false,children}:{title:string;open?:boolean;children:ReactNode}){return <details open={open}><summary>{title}</summary>{children}</details>}
export default function FlowEditor({flow,platform='twitch',ai,onSave,intro}:{flow:Flow;platform?:string;ai?:AIConfig;onSave:(f:Flow)=>Promise<void>;intro?:string}){
 const [counter,setCounter]=useState(!!flow.counter);const [timerSeconds,setTimerSeconds]=useState(flow.timerSeconds||300);
 const [sendType,setSendType]=useState(flow.sendType||'chat');const [sendColor,setSendColor]=useState(flow.sendColor||'primary');const [replyTo,setReplyTo]=useState(!!flow.replyTo);
 const [audio,setAudio]=useState(flow.audio||'');const [audioVolume,setAudioVolume]=useState(flow.audioVolume??1);
 const layout=flow.layout as {nodes?:Node<Data>[];edges?:Edge[]}|undefined;
 const startNodes:Node<Data>[]=layout?.nodes||[{id:'trigger',position:{x:60,y:140},data:{label:'⚡ '+(triggers[flow.trigger.kind]||flow.trigger.kind)},type:'input'},...flow.actions.map((action,i)=>({id:'action-'+i,position:{x:340+i*280,y:140},data:{label:actions[action.kind],action}}))];
 const startEdges:Edge[]=layout?.edges||flow.actions.map((_,i)=>({id:'e'+i,source:i===0?'trigger':'action-'+(i-1),target:'action-'+i,animated:true}));
 const [nodes,setNodes,onNodesChange]=useNodesState(startNodes);
 const [edges,setEdges,onEdgesChange]=useEdgesState(startEdges);
 const [selected,setSelected]=useState('trigger');const [name,setName]=useState(flow.name);const [trigger,setTrigger]=useState(flow.trigger);const [error,setError]=useState<FriendlyError|null>(null);const [busy,setBusy]=useState(false);
 const node=nodes.find(n=>n.id===selected);const action=node?.data.action;
 const connect=useCallback((c:Connection)=>setEdges(e=>addEdge({...c,animated:true},e)),[setEdges]);
 function update(a:Action){setNodes(ns=>ns.map(n=>n.id===selected?{...n,data:{label:actions[a.kind],action:a}}:n))}
 const keepText=(kind:string)=>["command","contains","voice","mention"].includes(kind);
function switchKind(current:Action,next:string){const base:Action={...current,kind:next};
  if(next==='ai.generate'&&!base.target.startsWith('local.'))base.target='local.aiResponse';
  if(next==='punish'){if(!['sender','first'].includes(base.target))base.target='sender';if(!base.punish)base.punish='timeout';if(base.value<1)base.value=60}
  if(next==='twitch'){if(!base.twOp)base.twOp='game';if(base.value<1)base.value=60}
  update(base)}
 async function save(){try{setBusy(true);setError(null);const ordered=orderedActions(nodes,edges);await onSave({...flow,name,trigger:trigger.kind==='timer'?{...trigger,permission:'everyone',cooldown:0,userCooldown:0,pattern:''}:keepText(trigger.kind)?trigger:{...trigger,pattern:''},counter:trigger.kind==='command'&&counter,timerSeconds,sendType,sendColor,replyTo,audio,audioVolume,actions:ordered,layout:{nodes,edges}})}catch(e){setError(friendlyError(e))}finally{setBusy(false)}}
 return <div className="flow-editor">
 {intro&&<p className="notice flow-intro">{intro}</p>}
 <div className="flow-toolbar"><input aria-label="Nome do fluxo" value={name} onChange={e=>setName(e.target.value)}/><button onClick={()=>{const id=crypto.randomUUID();setNodes(ns=>ns.concat({id,position:{x:300+ns.length*70,y:260},data:{label:actions.chat,action:newAction()}}));setSelected(id)}}><Plus size={16}/>Adicionar ação</button><button className="primary" disabled={busy} onClick={save}><Save size={16}/>Salvar fluxo</button></div>
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
 </Field>}{trigger.kind!=='timer'&&<>
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
 <Section title="Como sai" open>
 <Field label="Tipo de ação">
 <select value={action.kind} onChange={e=>switchKind(action,e.target.value)}>{Object.entries(actions).map(([k,n])=>
 <option key={k} value={k}>{n}</option>)}</select>
 </Field>{action.kind==='script'?<Field label="Script Rhai (retorna texto)">
 <textarea rows={5} value={action.text} onChange={e=>update({...action,text:e.target.value})}/>
 </Field>:action.kind==='variable.delete'?<p className="help">Escolha o valor que será apagado.</p>:action.kind==='twitch'&&!['game','title','timeout','ban','warn','mention'].includes(action.twOp||'game')?<p className="help">Esta operação não usa texto: o alvo e a duração vêm da fala ou dos campos abaixo.</p>:<MessageEditor key={selected} profileId={flow.profileId} label={action.kind.startsWith('ai')?'Como a IA deve responder':action.kind==='variable.increment'?'Quanto somar':action.kind==='punish'?'Motivo (vai para a Twitch e para o Histórico)':action.kind==='twitch'?((action.twOp||'game')==='game'?'Nome do jogo fixo (vazio usa o que você falou)':(action.twOp||'game')==='title'?'Novo título fixo (vazio usa o que você falou, até 140 caracteres)':(action.twOp||'game')==='mention'?'Mensagem (o bot marca @alvo na frente)':'Motivo (vai para a Twitch e para o Histórico)'):'Mensagem / conteúdo'} value={action.text} onChange={text=>update({...action,text})} flow={{...flow,name,trigger,actions:(()=>{try{return orderedActions(nodes,edges)}catch{return []}})()}}/>}{action.kind.startsWith('ai')&&<details className="ai-help"><summary>Como a IA monta a resposta</summary><p className="help">A mensagem atual, até 12 falas recentes dos últimos 5 minutos e as memórias entram automaticamente. Defina o tom aqui. {action.kind==='ai.generate'?'Esta ação só guarda a resposta. Conecte Enviar mensagem e clique em + Resposta da IA.':'Esta ação já envia ao chat e guarda a resposta para os próximos blocos.'}</p></details>}{ai&&<AIActionOptions action={action} ai={ai} onChange={update}/>}{action.kind==='ai.generate'&&<Field label="Nome da resposta" hint="Disponível somente nesta execução; evita misturar respostas entre pessoas.">
 <input value={(action.target||'local.aiResponse').replace(/^local\./,'')} onChange={e=>update({...action,target:'local.'+e.target.value})}/>
 </Field>}{action.kind.startsWith('ai')&&<AIResponseTest key={selected} profileId={flow.profileId} instruction={action.text}/>}{action.kind.startsWith('variable.')&&<VariableTarget value={action.target} onChange={target=>update({...action,target})}/>}{['memory','webhook'].includes(action.kind)&&<Field label={action.kind==='memory'?'Arquivo no vault':'Endereço HTTPS'}>
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
 </>}</Section>
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
