import {useEffect,useState} from 'react';
import {listen} from '@tauri-apps/api/event';
import {Plus,Search,Play,Workflow,Mic,Wand2} from 'lucide-react';
import {api,errorText,desktop} from './api';
import {newFlow,fmtDuration,type Flow,type Profile} from './types';
import {duplicateFlow} from './flow';
import {FOLLOW_COUNT_DEFAULT,FOLLOW_OK_DEFAULT,FOLLOW_SONG_DEFAULT,FOLLOW_WARN_DEFAULT,clippersCommandIssue,clippersFlow,commandConflict,donationCommandIssue,donationFlow,donationLinkIssue,followAgeFlow,followersCountFlow,nowPlayingFlow} from './commandModels';
import {Card,Empty,Field,Modal,Tag,Toggle} from './components';
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
 const [mKind,setMKind]=useState('followAge');
 const [mCommand,setMCommand]=useState('!followers');
 const [mAlias,setMAlias]=useState('!followage');
 const [mMention,setMMention]=useState(true);
 const [mLink,setMLink]=useState('');
 const [mText,setMText]=useState(FOLLOW_OK_DEFAULT);
 const [mWarn,setMWarn]=useState(FOLLOW_WARN_DEFAULT);
 const refreshCounts=()=>api<Record<string,number>>('command.counters',{profileId:profile.id}).then(setCounts).catch(e=>notify(errorText(e)));
 const afterChange=async()=>{refresh();refreshCounts();onChanged()};
 useEffect(()=>{void refreshCounts();if(!desktop)return;let cancelled=false;let unlisten:(()=>void)|undefined;void listen<{profileId:string;id:string;value:number}>('command-counter',({payload})=>{if(payload.profileId===profile.id)setCounts(c=>({...c,[payload.id]:payload.value}))}).then(fn=>{if(cancelled)fn();else unlisten=fn});return()=>{cancelled=true;unlisten?.()}},[profile.id]);
 useEffect(()=>{if(createToken)setEditing(newFlow(profile.id))},[createToken]);
 async function save(flow:Flow){await saveFlow(flow);await afterChange();notify(visual?'Automação salva.':'Comando salvo.')}
 /** Duplicar: cópia independente, já aberta no editor e desativada — a
  *  original não muda e nada dispara em dobro até você ativar a cópia. */
 async function duplicate(f:Flow){try{const copy=duplicateFlow(f);await saveFlow(copy);await afterChange();setEditing(copy);notify('Cópia criada desativada — ative quando estiver pronta.')}catch(e){notify(errorText(e))}}
 /** Comandos prontos: escolhe a função, confere nome e resposta e salva — o
  *  comando nasce pronto e ativado. Nome repetido é recusado, nunca sobrescrito. */
 async function createModel(){
  const pattern=(mKind==='followAge'?[mCommand,mAlias]:[mCommand]).join(', ');
  const conflito=commandConflict(flows,pattern);
  if(conflito){notify('Já existe o comando '+(mCommand.trim()||'')+" em '"+conflito+"'. Nada foi alterado — escolha outro nome ou edite o existente.");return}
  try{
   const base={command:mCommand,link:mLink};
   const flow=mKind==='followAge'
    ?followAgeFlow(profile.id,{command:mCommand,alias:mAlias,mention:mMention,ok:mText,warn:mWarn})
    :mKind==='followers'?followersCountFlow(profile.id,{command:mCommand,text:mText})
    :mKind==='song'?nowPlayingFlow(profile.id,{command:mCommand,text:mText})
    :mKind==='donation'?donationFlow(profile.id,base)
    :clippersFlow(profile.id,{command:mCommand});
   await saveFlow(flow);await afterChange();setModels(false);
   notify('Comando '+mCommand.trim().split(',')[0].trim()+' criado e ativado.');
  }catch(e){notify(errorText(e))}
 }
 /** Padrões de cada função: nasce pronto, editável antes de salvar. */
 function pickModel(kind:string){
  setMKind(kind);
  if(kind==='followAge'){setMCommand('!followers');setMAlias('!followage');setMMention(true);setMText(FOLLOW_OK_DEFAULT);setMWarn(FOLLOW_WARN_DEFAULT)}
  if(kind==='followers'){setMCommand('!seguidores');setMAlias('');setMText(FOLLOW_COUNT_DEFAULT)}
  if(kind==='song'){setMCommand('!musica');setMAlias('');setMText(FOLLOW_SONG_DEFAULT)}
  if(kind==='donation'){setMCommand('!pix');setMAlias('')}
  if(kind==='clippers'){setMCommand('!clip, !clipe');setMAlias('')}
 }
 const visible=flows.filter(f=>{
  if(f.trigger.kind==='timer')return false;
  if(!visual&&!(f.trigger.kind==='command'||f.actions.some(a=>a.kind==='ai.generate')))return false;
  return (f.name+' '+f.trigger.pattern).toLowerCase().includes(query.toLowerCase());
 });
 return <>
 <div className="section-tools"><div className="search"><Search size={17}/><input aria-label="Buscar comandos" value={query} onChange={e=>setQuery(e.target.value)} placeholder={visual?'Buscar automações…':'Buscar comandos…'}/></div><div className="row">{!visual&&onGoVoice&&<button onClick={onGoVoice}><Mic size={15}/>Comandos por voz</button>}<button onClick={()=>{pickModel('followAge');setModels(true)}} aria-haspopup="dialog"><Wand2 size={15}/>Comandos prontos</button><button onClick={()=>setConversationOpen(true)}>Resenha com IA</button><button onClick={()=>setVariablesOpen(true)}>Variáveis</button><button onClick={()=>setSimulate(true)}><Play size={15}/>Simular evento</button><button className="primary" onClick={()=>setEditing(newFlow(profile.id))}><Plus size={16}/>{visual?'Novo fluxo':'Criar comando personalizado'}</button></div></div>
 {visible.length?<Card><div className="table-scroll"><table><thead><tr><th>{visual?'Automação':'Comando'}</th><th>{visual?'Ações':'Resposta'}</th><th>Intervalo</th><th>Contador</th><th>Ativo</th><th></th></tr></thead><tbody>{visible.map(f=><tr key={f.id}><td><strong>{visual||f.trigger.kind==='timer'?f.name:f.trigger.pattern}</strong><small>{visual?f.trigger.pattern:f.name}</small></td><td className="response-cell">{visual?<Tag color="purple">{f.actions.length} ações conectadas</Tag>:f.actions[0]?.text}</td><td className="nowrap">{f.trigger.kind==='timer'?'A cada '+fmtDuration(f.timerSeconds||300):fmtDuration(f.trigger.cooldown)}</td><td>{f.counter?<button disabled={!desktop} onClick={()=>setCounterEdit(f)} aria-label={'Ajustar contador de '+f.name}>{counts[f.id]||0} · Ajustar</button>:'—'}</td><td><FlowEnableToggle flow={f} notify={notify} onSaved={afterChange}/></td><td><FlowRowActions flow={f} profile={profile} notify={notify} onEdit={()=>setEditing(f)} onRemove={()=>setRemove(f)} onDuplicate={()=>void duplicate(f)}/></td></tr>)}</tbody></table></div></Card>:<Empty title={query?'Nenhum resultado':'Sua primeira conversa começa aqui'} action={<button className="primary" onClick={()=>setEditing(newFlow(profile.id))}><Plus size={16}/>Criar {visual?'fluxo':'comando'}</button>}>{query?'Tente outro nome ou comando.':'Crie uma resposta de boas-vindas, compartilhe suas redes ou deixe a IA participar do chat.'}</Empty>}
 {visual&&<div className="tip"><Workflow size={18}/><p>Uma ideia, várias ações. Conecte um gatilho a mensagens, IA, memória e overlays na ordem que você quiser. A ordem é a das setas que você conectar, não a posição dos blocos na tela.</p></div>}
 {editing&&<FlowDialog key={editing.id} flow={editing} mode={visual?'flow':'command'} profile={profile} notify={notify} onSaved={afterChange} onClose={()=>setEditing(null)}/>}
 {remove&&<RemoveFlowDialog flow={remove} mode={visual?'flow':'command'} notify={notify} onClose={()=>setRemove(null)} onRemoved={afterChange}/>}
 {counterEdit&&<CounterAdjustDialog key={counterEdit.id} profile={profile} flow={counterEdit} value={counterEdit.id in counts?counts[counterEdit.id]:0} notify={notify} onClose={()=>setCounterEdit(null)} onSaved={()=>void refreshCounts()}/>}
 {simulate&&<SimulateDialog profile={profile} notify={notify} onClose={()=>setSimulate(false)}/>}
 {conversationOpen&&<Modal title="Criar resenha com IA" onClose={()=>setConversationOpen(false)}><ConversationForm profileId={profile.id} onSave={async f=>{await save(f);setConversationOpen(false)}}/></Modal>}
 {variablesOpen&&<VariableManager profileId={profile.id} onClose={()=>setVariablesOpen(false)}/>}
 {models&&<Modal title="Comandos prontos" onClose={()=>setModels(false)}><form className="form-pad" onSubmit={e=>{e.preventDefault();void createModel()}}>
  <p className="help">Escolha a função, confira o nome e a resposta e salve. O comando nasce pronto e ativado — sem montar automação nem criar variáveis.</p>
  <Field label="Função"><select value={mKind} onChange={e=>pickModel(e.target.value)}>
   <option value="followAge">Tempo de follow — desde quando a pessoa segue</option>
   <option value="followers">Total de seguidores</option>
   <option value="song">Música atual</option>
   <option value="donation">Link de doação (com resposta de IA)</option>
   <option value="clippers">Clipadores (!clip com contagem)</option>
  </select></Field>
  <Field label="Comando" hint="Aceita variações separadas por vírgula: !followers, !followage."><input value={mCommand} onChange={e=>setMCommand(e.target.value)}/></Field>
  {mKind==='followAge'&&<Field label="Alias (opcional)"><input value={mAlias} onChange={e=>setMAlias(e.target.value)}/></Field>}
  {mKind==='donation'&&<Field label="Seu link de doação" hint="Ex.: livepix.gg/seucanal. Vai no fim de toda resposta."><input value={mLink} onChange={e=>setMLink(e.target.value)} placeholder="livepix.gg/seucanal"/></Field>}
  {mKind==='followAge'&&<div className="switch-row"><div><strong>Consultar também @nome citado</strong><small>Com @usuario no chat, a consulta é da pessoa citada; sem ele, é de quem enviou.</small></div><Toggle label="Consultar também @nome citado" checked={mMention} onChange={setMMention}/></div>}
  {['followAge','followers','song'].includes(mKind)&&<Field label="Resposta" hint="Use os campos legíveis: nome do usuário, canal, tempo seguindo, data do follow."><textarea rows={3} value={mText} onChange={e=>setMText(e.target.value)}/></Field>}
  {mKind==='followAge'&&<Field label="Resposta quando não der" hint="Para quem não segue, não foi encontrado ou quando a consulta falha — a falha nunca vira “não segue”."><textarea rows={2} value={mWarn} onChange={e=>setMWarn(e.target.value)}/></Field>}
  <footer className="form-footer"><span className="help">Permissões, intervalos e edição avançada ficam no próprio comando, depois de salvo.</span><button type="button" onClick={()=>setModels(false)}>Cancelar</button><button className="primary"><Plus size={16}/>Salvar e ativar</button></footer>
 </form></Modal>}
 </>
}
