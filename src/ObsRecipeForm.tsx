import {Field} from './components';
import ObsActionFields,{ObsActionTest} from './ObsActionFields';
import {RewardPicker} from './FlowEditor';
import MessageEditor from './MessageEditor';
import type {Flow} from './types';
export default function ObsRecipeForm({flow,onChange,onAdvanced}:{flow:Flow;onChange:(f:Flow)=>void;onAdvanced:()=>void}){
 const index=flow.actions.findIndex(a=>a.kind==='obs'),chat=flow.actions.findIndex(a=>a.kind==='chat');
 const operations=flow.fnId==='recipe:obsText'?['set_text']:flow.fnId==='recipe:obsImage'?['show']:['filter_enable','filter_disable','filter_toggle'];
 return <div className="form-pad"><p className="help">Configure o resultado aqui. Este formulário edita a mesma automação do editor completo; seus ajustes continuam nela.</p>
  <Field label="Nome da automação"><input value={flow.name} onChange={e=>onChange({...flow,name:e.target.value})}/></Field>
  <h3>1. Quando acontece</h3>
  {flow.trigger.kind==='command'?<><Field label="Comando do chat"><input value={flow.trigger.pattern} onChange={e=>onChange({...flow,trigger:{...flow.trigger,pattern:e.target.value}})}/></Field><p className="help">Exemplo: !tela Boa noite! Sem texto depois do comando, a receita não altera a fonte.</p><Field label="Quem pode usar"><select value={flow.trigger.permission} onChange={e=>onChange({...flow,trigger:{...flow.trigger,permission:e.target.value}})}><option value="broadcaster">Só o streamer</option><option value="moderator">Moderadores e streamer</option><option value="everyone">Público geral</option><option value="subscriber">Assinantes</option></select></Field></>:<><RewardPicker profileId={flow.profileId} value={flow.trigger.pattern} onChange={(pattern,patternName)=>onChange({...flow,trigger:{...flow.trigger,pattern,patternName}})}/><p className="help">Escolha uma recompensa específica antes de salvar esta receita.</p></>}
  <Field label="Intervalo entre usos (segundos)"><input type="number" min={0} max={86400} value={flow.trigger.cooldown} onChange={e=>onChange({...flow,trigger:{...flow.trigger,cooldown:Math.max(0,Number(e.target.value)||0)}})}/></Field>
  <h3>2. O que aparece no OBS</h3>
  <ObsActionFields profileId={flow.profileId} action={flow.actions[index]} onChange={a=>onChange({...flow,actions:flow.actions.map((v,i)=>i===index?a:v)})} flow={flow} at={index} operations={operations}/>
  <ObsActionTest profileId={flow.profileId} action={flow.actions[index]}/>
  <h3>3. Confirmação no chat</h3>
  <MessageEditor profileId={flow.profileId} label="Mensagem de confirmação" value={flow.actions[chat].text} onChange={text=>onChange({...flow,actions:flow.actions.map((a,i)=>i===chat?{...a,text}:a)})} flow={flow} at={chat}/>
  <p className="help">A confirmação só é enviada depois que o OBS aceita a operação. O teste do fluxo ensaia a sequência sem alterar o OBS real. Para conferir a fonte ao vivo, use Testar ação em OBS Studio.</p>
  <button type="button" onClick={onAdvanced}>Editar etapas no editor completo</button>
 </div>;
}
