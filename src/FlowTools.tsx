import {useEffect,useState} from 'react';
import {Save,Play,Trash2,Pencil,Package} from 'lucide-react';
import {api,errorText,desktop} from './api';
import {type Flow,type Profile,type Preset} from './types';
import {Modal,Field,Toggle} from './components';
import FlowEditor from './FlowEditor';
import MessageEditor from './MessageEditor';
import CommandOptions from './CommandOptions';

export type FlowMode='command'|'timer'|'flow';
type Notify=(s:string)=>void;

export function useFlows(profileId:string,notify:Notify){
 const [flows,setFlows]=useState<Flow[]>([]);
 const refresh=()=>api<Flow[]>('flows',{profileId}).then(setFlows).catch(e=>notify(errorText(e)));
 useEffect(()=>{void refresh()},[profileId]);
 return {flows,refresh};
}

export async function saveFlow(flow:Flow){
 if(!flow.name.trim()||!flow.trigger.pattern&&flow.trigger.kind==='command')throw Error('Preencha o nome e o comando.');
 await api('flow.save',{flow});
}

export function FlowDialog({flow,mode,profile,notify,onSaved,onClose}:{flow:Flow;mode:FlowMode;profile:Profile;notify:Notify;onSaved:()=>Promise<void>|void;onClose:()=>void}){
 const [editing,setEditing]=useState<Flow>(flow);
 const editor=mode==='flow'||editing.actions.length>1;
 const title=mode==='flow'?'Editor de automação':mode==='timer'?'Configurar timer':'Configurar comando';
 async function save(next:Flow){
  await saveFlow(next);
  await onSaved();
  notify('Automação salva.');
  onClose();
 }
 return <Modal wide={editor} title={title} onClose={onClose}>
 {editor?<FlowEditor flow={editing} onSave={save}/>:<form className="form-pad" onSubmit={e=>{e.preventDefault();save(editing).catch(err=>notify(errorText(err)))}}>
  <div className="form-grid">
   <Field label="Nome"><input required value={editing.name} onChange={e=>setEditing({...editing,name:e.target.value})}/></Field>
   {mode!=='timer'&&<Field label="Comando"><input required pattern="![^\s]+" value={editing.trigger.pattern} onChange={e=>setEditing({...editing,trigger:{...editing.trigger,pattern:e.target.value}})}/></Field>}
  </div>
  <MessageEditor profileId={profile.id} label="Resposta" required value={editing.actions[0].text} onChange={text=>setEditing({...editing,actions:[{...editing.actions[0],text}]})}/>
  <CommandOptions timer={mode==='timer'} seconds={editing.timerSeconds??300} counter={!!editing.counter} onSeconds={timerSeconds=>setEditing({...editing,timerSeconds})} onCounter={counter=>setEditing({...editing,counter})}/>
  {editing.counter&&<button type="button" onClick={()=>setEditing({...editing,actions:[{...editing.actions[0],text:editing.actions[0].text+'{{commandCount}}'}]})}>+ Contagem do comando</button>}
  {mode!=='timer'&&<div className="form-grid">
   <Field label="Quem pode usar"><select value={editing.trigger.permission} onChange={e=>setEditing({...editing,trigger:{...editing.trigger,permission:e.target.value}})}><option value="everyone">Todo mundo</option><option value="subscriber">Assinantes</option><option value="moderator">Moderadores</option><option value="broadcaster">Só o streamer</option></select></Field>
   <Field label="Intervalo entre usos (segundos)"><input type="number" min="0" max="86400" value={editing.trigger.cooldown} onChange={e=>setEditing({...editing,trigger:{...editing.trigger,cooldown:+e.target.value}})}/></Field>
   <Field label="Intervalo por pessoa (segundos)"><input type="number" min="0" max="86400" value={editing.trigger.userCooldown} onChange={e=>setEditing({...editing,trigger:{...editing.trigger,userCooldown:+e.target.value}})}/></Field>
  </div>}
  <footer className="form-footer"><span className="help">{mode==='timer'?'Ao pausar, desconectar ou editar, o intervalo começa novamente.':'Você pode evoluir este comando no editor visual.'}</span><button className="primary"><Save size={16}/>{mode==='timer'?'Salvar timer':'Salvar comando'}</button></footer>
 </form>}
 </Modal>
}

export function RemoveFlowDialog({flow,mode,onClose,onRemoved,notify}:{flow:Flow;mode:FlowMode;onClose:()=>void;onRemoved:()=>Promise<void>|void;notify:Notify}){
 const title=mode==='timer'?'Apagar timer?':mode==='command'?'Apagar comando?':'Apagar automação?';
 return <Modal title={title} onClose={onClose}><div className="form-pad"><p>“{flow.name}” será removida deste perfil.</p><footer className="form-footer"><button onClick={onClose}>Cancelar</button><button className="danger" onClick={async()=>{try{await api('flow.delete',{profileId:flow.profileId,id:flow.id});onClose();await onRemoved()}catch(e){notify(errorText(e))}}}>Apagar</button></footer></div></Modal>
}

export function SimulateDialog({profile,notify,onClose}:{profile:Profile;notify:Notify;onClose:()=>void}){
 const [message,setMessage]=useState('!oi');const [role,setRole]=useState('everyone');
 return <Modal title="Simular evento de chat" onClose={onClose}><div className="form-pad"><p className="help">O teste não publica mensagens, não chama serviços e não altera memórias ou pontos. Acompanhe a sequência no histórico.</p><Field label="Mensagem"><input value={message} onChange={e=>setMessage(e.target.value)}/></Field><Field label="Permissão do espectador"><select value={role} onChange={e=>setRole(e.target.value)}><option value="everyone">Todo mundo</option><option value="subscriber">Assinante</option><option value="moderator">Moderador</option><option value="broadcaster">Streamer</option></select></Field><button className="primary" onClick={async()=>{try{await api('simulate',{event:{id:crypto.randomUUID(),profileId:profile.id,kind:'chat',user:'Espectador de teste',userId:'test-user',role,message,data:{},simulated:true}});notify('Simulação enviada. Veja o histórico.');onClose()}catch(e){notify(errorText(e))}}}><Play size={16}/>Executar simulação</button></div></Modal>
}

export function CounterAdjustDialog({profile,flow,value,onClose,onSaved,notify}:{profile:Profile;flow:Flow;value:number;onClose:()=>void;onSaved:()=>void;notify:Notify}){
 const [counterValue,setCounterValue]=useState(value);
 return <Modal title={'Ajustar contador de '+flow.name} onClose={onClose}><form className="form-pad" onSubmit={async e=>{e.preventDefault();try{await api('command.counter.set',{profileId:profile.id,id:flow.id,value:counterValue});onSaved();onClose();notify('Contador ajustado.')}catch(err){notify(errorText(err))}}}><p>Define o total deste comando. Os outros contadores não mudam. Use zero para reiniciar; o próximo uso contará 1.</p><Field label="Novo total"><input type="number" required min="0" max="9007199254740991" step="1" value={counterValue} onChange={e=>setCounterValue(Number(e.target.value))}/></Field><button type="submit" className="primary">Confirmar novo total</button></form></Modal>
}

export function FlowEnableToggle({flow,notify,onSaved}:{flow:Flow;notify:Notify;onSaved:()=>Promise<void>|void}){
 return <Toggle checked={flow.enabled} label={'Ativar '+flow.name} onChange={async enabled=>{try{await api('flow.save',{flow:{...flow,enabled}});await onSaved()}catch(e){notify(errorText(e))}}}/>
}

export function FlowRowActions({flow,profile,notify,onEdit,onRemove}:{flow:Flow;profile:Profile;notify:Notify;onEdit:()=>void;onRemove:()=>void}){
 return <div className="row">
 {flow.trigger.kind==='timer'&&<button className="icon-button" disabled={!desktop||!flow.enabled} aria-label={'Simular timer '+flow.name} onClick={()=>api('timer.preview',{profileId:profile.id,id:flow.id}).then(()=>notify('Simulação do timer enviada. Veja o Histórico; nada foi publicado.')).catch(e=>notify(errorText(e)))}><Play size={16}/></button>}
 <button className="icon-button" aria-label={'Salvar preset '+flow.name} onClick={async()=>{try{const preset=await api<Preset>('preset.create',{profileId:profile.id,kind:flow.trigger.kind==='command'?'command':'flow',name:flow.name,ids:[flow.id]});await api('preset.save',{preset});notify('Salvo na biblioteca de presets.')}catch(e){notify(errorText(e))}}}><Package size={16}/></button>
 <button className="icon-button" aria-label={'Editar '+flow.name} onClick={onEdit}><Pencil size={16}/></button>
 <button className="icon-button danger" aria-label={'Apagar '+flow.name} onClick={onRemove}><Trash2 size={16}/></button>
 </div>
}
