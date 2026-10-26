import {useEffect,useState} from 'react';
import {listen} from '@tauri-apps/api/event';
import {Plus,Search,Play,Workflow,Mic,Wand2} from 'lucide-react';
import {api,errorText,desktop} from './api';
import {newFlow,fmtDuration,type Flow,type Profile} from './types';
import {duplicateFlow} from './flow';
import RecipeCatalog from './RecipeCatalog';
import {recipeDraft,recipeConfiguration,type RecipeId} from './recipes';
import type {FlowMode} from './FlowTools';
import {Card,Empty,Modal,Tag} from './components';
import {CounterAdjustDialog,FlowDialog,FlowEnableToggle,FlowRowActions,RemoveFlowDialog,SimulateDialog,saveFlow,useFlows} from './FlowTools';
import {VariableManager} from './Variables';
import {ConversationForm} from './AIConversation';

export default function Commands({profile,visual,notify,onChanged,createToken,onGoVoice,onNavigate}:{profile:Profile;visual:boolean;notify:(s:string)=>void;onChanged:()=>void;createToken:number;onGoVoice?:()=>void;onNavigate?:(page:string,section?:string)=>void}){
 const {flows,refresh}=useFlows(profile.id,notify);
 const [counts,setCounts]=useState<Record<string,number>>({});
 const [query,setQuery]=useState('');
 const [editing,setEditing]=useState<Flow|null>(null);
 const [remove,setRemove]=useState<Flow|null>(null);
 const [counterEdit,setCounterEdit]=useState<Flow|null>(null);
 const [simulate,setSimulate]=useState(false);
 const [variablesOpen,setVariablesOpen]=useState(false);
 const [conversationOpen,setConversationOpen]=useState(false);
 const [recipesOpen,setRecipesOpen]=useState(false);
 const [editingMode,setEditingMode]=useState<FlowMode>(visual?'flow':'command');
 const [recipeValidation,setRecipeValidation]=useState<RecipeId|undefined>();
 const openFlow=(flow:Flow,mode:FlowMode=visual?'flow':'command',recipe?:RecipeId)=>{setEditingMode(mode);setRecipeValidation(recipe);setEditing(flow)};
 const [voiceCmds,setVoiceCmds]=useState<{command:string;label:string}[]>([]);
 useEffect(()=>{if(!desktop)return;api<{config?:{bindings?:{kind:string;command:string;rewardName?:string}[]}}>('voiceResgate.get',{profileId:profile.id}).then(r=>setVoiceCmds((r?.config?.bindings||[]).filter(b=>b.kind==='command'&&b.command).map(b=>({command:b.command.trim(),label:b.rewardName||'Troca de voz'})))).catch(()=>{})},[profile.id]);
 const moduleCmds:Record<string,string[]>={points:['!pontos','!ranking','!transferir','!loja','!resgatar'],songs:['!musica','!minhamusica','!removermusica','!musicas'],raffles:['!ticket'],games:['!roleta','!duelo','!aceitar','!bingo']};
 const reservedCmds:Record<string,string>={};Object.entries(moduleCmds).forEach(([m,lista])=>{if((profile as unknown as {modules?:Record<string,boolean>}).modules?.[m]){lista.forEach(x=>{reservedCmds[x]='o módulo '+m})}});voiceCmds.forEach(v=>{reservedCmds[v.command]=`“${v.label}” da tela Voicemod`});
 const refreshCounts=()=>api<Record<string,number>>('command.counters',{profileId:profile.id}).then(setCounts).catch(e=>notify(errorText(e)));
 const afterChange=async()=>{refresh();refreshCounts();onChanged()};
 useEffect(()=>{void refreshCounts();if(!desktop)return;let cancelled=false;let unlisten:(()=>void)|undefined;void listen<{profileId:string;id:string;value:number}>('command-counter',({payload})=>{if(payload.profileId===profile.id)setCounts(c=>({...c,[payload.id]:payload.value}))}).then(fn=>{if(cancelled)fn();else unlisten=fn});return()=>{cancelled=true;unlisten?.()}},[profile.id]);
 useEffect(()=>{if(createToken)openFlow(newFlow(profile.id))},[createToken]);
 async function save(flow:Flow){await saveFlow(flow);await afterChange();notify(visual?'Automação salva.':'Comando salvo.')}
 /** Duplicar: cópia independente, já aberta no editor e desativada — a
  *  original não muda e nada dispara em dobro até você ativar a cópia. */
 async function duplicate(f:Flow){try{const copy=duplicateFlow(f);await saveFlow(copy);await afterChange();openFlow(copy);notify('Cópia criada desativada — ative quando estiver pronta.')}catch(e){notify(errorText(e))}}
 async function pickRecipe(id:RecipeId){
  if(id==='voice'||id==='raid'){
   if(!onNavigate)throw new Error('Não foi possível abrir o painel desta receita.');
   onNavigate(id==='voice'?'vozfx':'community',id==='voice'?'etapa3':'shoutout');setRecipesOpen(false);return;
  }
  const draft=recipeDraft(id,profile.id);
  const latest=await api<Flow[]>('flows',{profileId:profile.id});
  if(!Array.isArray(latest))throw new Error('Não foi possível conferir as configurações existentes. Tente novamente.');
  const existing=recipeConfiguration(id,profile.id,latest);
  if(id==='conversation'&&!existing){setRecipesOpen(false);setConversationOpen(true);return}
  const flow=existing||draft;
  if(!flow)return;
  openFlow(flow,flow.trigger.kind==='timer'?'timer':flow.trigger.kind==='command'?'command':'flow',existing?undefined:id);
  setRecipesOpen(false);
 }
 const visible=flows.filter(f=>{
  if(f.trigger.kind==='timer')return false;
  if(!visual&&!(f.trigger.kind==='command'||(f.actions.some(a=>a.kind==='ai.generate')&&['chat','contains','mention'].includes(f.trigger.kind))))return false;
  return (f.name+' '+f.trigger.pattern).toLowerCase().includes(query.toLowerCase());
 });
 return <>
 <div className="section-tools"><div className="search"><Search size={17}/><input aria-label="Buscar comandos" value={query} onChange={e=>setQuery(e.target.value)} placeholder={visual?'Buscar automações…':'Buscar comandos…'}/></div><div className="row">{!visual&&onGoVoice&&<button onClick={onGoVoice}><Mic size={15}/>Comandos por voz</button>}<button onClick={()=>setRecipesOpen(true)} aria-haspopup="dialog"><Wand2 size={15}/>Receitas prontas</button><button onClick={()=>setVariablesOpen(true)}>Variáveis</button><button onClick={()=>setSimulate(true)}><Play size={15}/>Simular evento</button><button className="primary" onClick={()=>openFlow(newFlow(profile.id))}><Plus size={16}/>{visual?'Novo fluxo':'Criar comando personalizado'}</button></div></div>
 {visible.length?<Card><div className="table-scroll"><table><thead><tr><th>{visual?'Automação':'Comando'}</th><th>{visual?'Ações':'Resposta'}</th><th>Intervalo</th><th>Contador</th><th>Ativo</th><th></th></tr></thead><tbody>{visible.map(f=><tr key={f.id}><td><strong>{visual||f.trigger.kind==='timer'?f.name:f.trigger.pattern}</strong><small>{visual?f.trigger.pattern:f.name}</small></td><td className="response-cell">{visual?<Tag color="purple">{f.actions.length} ações conectadas</Tag>:f.actions[0]?.text}</td><td className="nowrap">{f.trigger.kind==='timer'?'A cada '+fmtDuration(f.timerSeconds||300):fmtDuration(f.trigger.cooldown)}</td><td>{f.counter?<button disabled={!desktop} onClick={()=>setCounterEdit(f)} aria-label={'Ajustar contador de '+f.name}>{counts[f.id]||0} · Ajustar</button>:'—'}</td><td><FlowEnableToggle flow={f} notify={notify} onSaved={afterChange}/></td><td><FlowRowActions flow={f} profile={profile} notify={notify} onEdit={()=>openFlow(f)} onRemove={()=>setRemove(f)} onDuplicate={()=>void duplicate(f)}/></td></tr>)}</tbody></table></div></Card>:<Empty title={query?'Nenhum resultado':'Sua primeira conversa começa aqui'} action={<button className="primary" onClick={()=>openFlow(newFlow(profile.id))}><Plus size={16}/>Criar {visual?'fluxo':'comando'}</button>}>{query?'Tente outro nome ou comando.':'Crie uma resposta de boas-vindas, compartilhe suas redes ou deixe a IA participar do chat.'}</Empty>}
 {(!visual&&Object.keys(reservedCmds).length>0)&&<details className="tip"><summary>Outros comandos do chat (módulos e voz)</summary><div className="table-scroll"><table><thead><tr><th>Comando</th><th>Função</th><th>Origem</th></tr></thead><tbody>{Object.entries(reservedCmds).map(([nome,origem])=><tr key={nome}><td><strong>{nome}</strong></td><td>{origem}</td><td><small>{origem.includes('Voicemod')?'Voicemod':'Módulo da comunidade'}</small></td></tr>)}</tbody></table></div><p className="help">Um comando com o mesmo nome é recusado na criação, para não brigar com estes. A precedência atual: comandos prontos da comunidade respondem antes dos seus; entre os seus, respostas por TXT processam antes das automações.</p></details>}
 {visual&&<div className="tip"><Workflow size={18}/><p>Uma ideia, várias ações. Conecte um gatilho a mensagens, IA, memória e overlays na ordem que você quiser. A ordem é a das setas que você conectar, não a posição dos blocos na tela.</p></div>}
 {editing&&<FlowDialog key={editing.id} flow={editing} mode={editingMode} profile={profile} notify={notify} reservedCommands={reservedCmds} requireSoundReward={recipeValidation==='sound'} requireDonationLink={recipeValidation==='donation'} onSaved={async()=>{await afterChange();if(editingMode==='timer')onNavigate?.('timers');else if(editingMode==='flow'&&!visual)onNavigate?.('flows')}} onClose={()=>setEditing(null)}/>}
 {remove&&<RemoveFlowDialog flow={remove} mode={visual?'flow':'command'} notify={notify} onClose={()=>setRemove(null)} onRemoved={afterChange}/>}
 {counterEdit&&<CounterAdjustDialog key={counterEdit.id} profile={profile} flow={counterEdit} value={counterEdit.id in counts?counts[counterEdit.id]:0} notify={notify} onClose={()=>setCounterEdit(null)} onSaved={()=>void refreshCounts()}/>}
 {simulate&&<SimulateDialog profile={profile} notify={notify} onClose={()=>setSimulate(false)}/>}
 {conversationOpen&&<Modal title="Criar resenha com IA" onClose={()=>setConversationOpen(false)}><ConversationForm profileId={profile.id} onSave={async f=>{await save({...f,enabled:false,fnId:'recipe:conversation',fnVersion:1});setConversationOpen(false)}}/></Modal>}
 {variablesOpen&&<VariableManager profileId={profile.id} onClose={()=>setVariablesOpen(false)}/>}
 {recipesOpen&&<RecipeCatalog profileId={profile.id} flows={flows} onPick={pickRecipe} onClose={()=>setRecipesOpen(false)}/>}
 </>
}
