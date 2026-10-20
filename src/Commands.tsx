import {useEffect,useState} from 'react';
import {listen} from '@tauri-apps/api/event';
import {Plus,Search,Play,Workflow,Mic,Wand2} from 'lucide-react';
import {api,errorText,desktop} from './api';
import {newFlow,fmtDuration,type Flow,type Profile} from './types';
import {duplicateFlow} from './flow';
import {clippersCommandIssue,clippersFlow,donationCommandIssue,donationFlow,donationLinkIssue} from './commandModels';
import {Card,Empty,Field,Modal,Tag} from './components';
import {CounterAdjustDialog,FlowDialog,FlowEnableToggle,FlowRowActions,RemoveFlowDialog,SimulateDialog,saveFlow,useFlows} from './FlowTools';
import {VariableManager} from './Variables';
import {ConversationForm} from './AIConversation';

export default function Commands({profile,visual,notify,onChanged,createToken,onGoVoice}:{profile:Profile;visual:boolean;notify:(s:string)=>void;onChanged:()=>void;createToken:number;onGoVoice?:()=>void}){
 const {flows,refresh}=useFlows(profile.id,notify);
 const [counts,setCounts]=useState<Record<string,number>>({});
 const [query,setQuery]=useState('');
 const [editing,setEditing]=useState<Flow|null>(null);
 const [remove,setRemove]=useState<Flow|null>(null);
 const [counterEdit,setCounterEdit]=useState<Flow|null>(null);
 const [simulate,setSimulate]=useState(false);
 const [variablesOpen,setVariablesOpen]=useState(false);
 const [conversationOpen,setConversationOpen]=useState(false);
 const [models,setModels]=useState(false);
 const [mCommand,setMCommand]=useState('!pix');
 const [mLink,setMLink]=useState('');
 const [cCommand,setCCommand]=useState('!clip, !clipe');
 const refreshCounts=()=>api<Record<string,number>>('command.counters',{profileId:profile.id}).then(setCounts).catch(e=>notify(errorText(e)));
 const afterChange=async()=>{refresh();refreshCounts();onChanged()};
 useEffect(()=>{void refreshCounts();if(!desktop)return;let cancelled=false;let unlisten:(()=>void)|undefined;void listen<{profileId:string;id:string;value:number}>('command-counter',({payload})=>{if(payload.profileId===profile.id)setCounts(c=>({...c,[payload.id]:payload.value}))}).then(fn=>{if(cancelled)fn();else unlisten=fn});return()=>{cancelled=true;unlisten?.()}},[profile.id]);
 useEffect(()=>{if(createToken)setEditing(newFlow(profile.id))},[createToken]);
 async function save(flow:Flow){await saveFlow(flow);await afterChange();notify('Automação salva.')}
 /** Duplicar: cópia independente, já aberta no editor e desativada — a
  *  original não muda e nada dispara em dobro até você ativar a cópia. */
 async function duplicate(f:Flow){try{const copy=duplicateFlow(f);await saveFlow(copy);await afterChange();setEditing(copy);notify('Cópia criada desativada — ative quando estiver pronta.')}catch(e){notify(errorText(e))}}
 /** Comando pronto de doação: nasce ativado, com a frase da IA variada e o
  *  link da pessoa no fim de toda resposta — só falta digitar o comando. */
 async function createDonation(){
  const issue=donationCommandIssue(mCommand)||donationLinkIssue(mLink);
  if(issue){notify(issue);return}
  try{const flow=donationFlow(profile.id,{command:mCommand,link:mLink});await saveFlow(flow);await afterChange();setModels(false);notify('Comando '+mCommand.trim().split(',')[0].trim()+' criado e ativado. Digite no chat para testar.')}catch(e){notify(errorText(e))}
 }
 /** Comando pronto dos clipadores: cria o clipe na hora, soma na contagem de
  *  quem pediu e anuncia o número — a contagem fica em {{user.clips}}. */
 async function createClippers(){
  const issue=clippersCommandIssue(cCommand);
  if(issue){notify(issue);return}
  try{const flow=clippersFlow(profile.id,{command:cCommand});await saveFlow(flow);await afterChange();setModels(false);notify('Comando '+cCommand.trim().split(',')[0].trim()+' criado e ativado. Cada clipe criado soma na contagem de quem pediu.')}catch(e){notify(errorText(e))}
 }
 const visible=flows.filter(f=>{
  if(f.trigger.kind==='timer')return false;
  if(!visual&&!(f.trigger.kind==='command'||f.actions.some(a=>a.kind==='ai.generate')))return false;
  return (f.name+' '+f.trigger.pattern).toLowerCase().includes(query.toLowerCase());
 });
 return <>
 <div className="section-tools"><div className="search"><Search size={17}/><input aria-label="Buscar comandos" value={query} onChange={e=>setQuery(e.target.value)} placeholder={visual?'Buscar automações…':'Buscar comandos…'}/></div><div className="row">{!visual&&onGoVoice&&<button onClick={onGoVoice}><Mic size={15}/>Comandos por voz</button>}<button onClick={()=>setModels(true)} aria-haspopup="dialog"><Wand2 size={15}/>Modelos</button><button onClick={()=>setConversationOpen(true)}>Resenha com IA</button><button onClick={()=>setVariablesOpen(true)}>Variáveis</button><button onClick={()=>setSimulate(true)}><Play size={15}/>Simular evento</button><button className="primary" onClick={()=>setEditing(newFlow(profile.id))}><Plus size={16}/>{visual?'Novo fluxo':'Novo comando'}</button></div></div>
 {visible.length?<Card><div className="table-scroll"><table><thead><tr><th>{visual?'Automação':'Comando'}</th><th>{visual?'Ações':'Resposta'}</th><th>Intervalo</th><th>Contador</th><th>Ativo</th><th></th></tr></thead><tbody>{visible.map(f=><tr key={f.id}><td><strong>{visual||f.trigger.kind==='timer'?f.name:f.trigger.pattern}</strong><small>{visual?f.trigger.pattern:f.name}</small></td><td className="response-cell">{visual?<Tag color="purple">{f.actions.length} ações conectadas</Tag>:f.actions[0]?.text}</td><td className="nowrap">{f.trigger.kind==='timer'?'A cada '+fmtDuration(f.timerSeconds||300):fmtDuration(f.trigger.cooldown)}</td><td>{f.counter?<button disabled={!desktop} onClick={()=>setCounterEdit(f)} aria-label={'Ajustar contador de '+f.name}>{counts[f.id]||0} · Ajustar</button>:'—'}</td><td><FlowEnableToggle flow={f} notify={notify} onSaved={afterChange}/></td><td><FlowRowActions flow={f} profile={profile} notify={notify} onEdit={()=>setEditing(f)} onRemove={()=>setRemove(f)} onDuplicate={()=>void duplicate(f)}/></td></tr>)}</tbody></table></div></Card>:<Empty title={query?'Nenhum resultado':'Sua primeira conversa começa aqui'} action={<button className="primary" onClick={()=>setEditing(newFlow(profile.id))}><Plus size={16}/>Criar {visual?'fluxo':'comando'}</button>}>{query?'Tente outro nome ou comando.':'Crie uma resposta de boas-vindas, compartilhe suas redes ou deixe a IA participar do chat.'}</Empty>}
 {visual&&<div className="tip"><Workflow size={18}/><p>Uma ideia, várias ações. Conecte um gatilho a mensagens, IA, memória e overlays na ordem que você quiser. A ordem é a das setas que você conectar, não a posição dos blocos na tela.</p></div>}
 {editing&&<FlowDialog key={editing.id} flow={editing} mode={visual?'flow':'command'} profile={profile} notify={notify} onSaved={afterChange} onClose={()=>setEditing(null)}/>}
 {remove&&<RemoveFlowDialog flow={remove} mode={visual?'flow':'command'} notify={notify} onClose={()=>setRemove(null)} onRemoved={afterChange}/>}
 {counterEdit&&<CounterAdjustDialog key={counterEdit.id} profile={profile} flow={counterEdit} value={counterEdit.id in counts?counts[counterEdit.id]:0} notify={notify} onClose={()=>setCounterEdit(null)} onSaved={()=>void refreshCounts()}/>}
 {simulate&&<SimulateDialog profile={profile} notify={notify} onClose={()=>setSimulate(false)}/>}
 {conversationOpen&&<Modal title="Criar resenha com IA" onClose={()=>setConversationOpen(false)}><ConversationForm profileId={profile.id} onSave={async f=>{await save(f);setConversationOpen(false)}}/></Modal>}
 {variablesOpen&&<VariableManager profileId={profile.id} onClose={()=>setVariablesOpen(false)}/>}
 {models&&<Modal title="Comandos prontos" onClose={()=>setModels(false)}><form className="form-pad" onSubmit={e=>{e.preventDefault();void createDonation()}}><h3>Link de doação</h3><p className="help">Resposta de IA diferente a cada chamada, com seu link de doação fixo no fim de toda resposta. Precisa da IA configurada em Personalidade de IA.</p><Field label="Comando" hint="Separe variações com vírgula: !pix, !doar."><input value={mCommand} onChange={e=>setMCommand(e.target.value)}/></Field><Field label="Seu link de doação" hint="Ex.: livepix.gg/seucanal. Vai no fim de toda resposta."><input value={mLink} onChange={e=>setMLink(e.target.value)} placeholder="livepix.gg/seucanal"/></Field><footer className="form-footer"><button type="button" onClick={()=>setModels(false)}>Cancelar</button><button className="primary"><Plus size={16}/>Criar comando</button></footer></form><form className="form-pad" onSubmit={e=>{e.preventDefault();void createClippers()}}><h3>Clipadores (!clip)</h3><p className="help">Cria o clipe da live na hora, conta de quem foi cada clipe e anuncia o número de quem pediu (fica em {'{{user.clips}}'}, pronta para ranking). Só conta clipe criado com sucesso.</p><Field label="Comando" hint="Separe variações com vírgula: !clip, !clipe."><input value={cCommand} onChange={e=>setCCommand(e.target.value)}/></Field><footer className="form-footer"><span className="help">O link do clipe sai na própria resposta.</span><button className="primary"><Plus size={16}/>Criar comando</button></footer></form></Modal>}
 </>
}
