import {useRef} from 'react';
import {useAutoGrow} from './autoGrow';
import {Field} from './components';
import {VariableHelp} from './Variables';
import {insertVariable,messageParts,missingLocals} from './variableCatalog';
import type {Flow} from './types';

export default function MessageEditor({profileId,label,value,onChange,flow,required=false}:{profileId:string;label:string;value:string;onChange:(v:string)=>void;flow?:Flow;required?:boolean}){
 const ref=useRef<HTMLTextAreaElement>(null);const selection=useRef<{start:number;end:number}|null>(null);
 function insert(token:string){
  // O texto e o cursor vêm do textarea vivo, não do valor do último render:
  // quem digita e clica na variável no mesmo instante não perde o texto nem
  // duplica o marcador por causa de um render atrasado.
  const el=ref.current;const base=el?el.value:value;
  const range=(selection.current&&el&&el.selectionStart!=null&&el.selectionEnd!=null)?{start:el.selectionStart,end:el.selectionEnd}:(selection.current||{start:base.length,end:base.length});
  const next=insertVariable(base,token,range.start,range.end);onChange(next.text);selection.current={start:next.caret,end:next.caret};
  // Devolve o cursor só se a mensagem ainda é a que acabou de entrar: um
  // clique ou um preenchimento posterior não podem ter o cursor roubado.
  requestAnimationFrame(()=>{const t=ref.current;if(!t||t.value!==next.text)return;t.focus();t.setSelectionRange(next.caret,next.caret)})}
 const parts=messageParts(value);const missing=flow?missingLocals(value,flow.actions):[];useAutoGrow(ref,value);
 return <div className="message-editor"><Field label={label} hint="Posicione o cursor e escolha uma informação nos botões abaixo."><textarea ref={ref} className="auto-grow" required={required} rows={4} value={value} onSelect={e=>{selection.current={start:e.currentTarget.selectionStart,end:e.currentTarget.selectionEnd}}} onChange={e=>onChange(e.target.value)} placeholder="Escreva sua mensagem…"/></Field>
 {parts.some(p=>p.label)&&<div className="message-readable"><small>Sua mensagem com os nomes das informações</small><p>{parts.map((p,i)=>p.label?<span className="variable-chip" key={i} title={p.text}>{p.label}</span>:<span key={i}>{p.text}</span>)}</p></div>}
 {missing.length>0&&<p className="inline-warning" role="status">Nenhuma ação deste fluxo define <code>{missing.join(', ')}</code>. No Histórico esta ação para com “Variável ausente” e a mensagem não sai. Acrescente a ação que gera o valor ou escreva <code>{missing[0]}|default:0</code> no próprio código da variável.</p>}
 <div className="variable-shortcuts" aria-label="Inserção rápida"><button type="button" onClick={()=>insert('{{local.aiResponse}}')}>+ Resposta da IA</button><button type="button" onClick={()=>insert('{{message}}')}>+ Mensagem do chat</button><button type="button" onClick={()=>insert('{{user}}')}>+ Nome da pessoa</button><button type="button" onClick={()=>insert('{{channel}}')}>+ Nome do canal</button><button type="button" onClick={()=>insert('{{rawInput}}')}>+ Texto do pedido</button></div>
 <VariableHelp profileId={profileId} text={value} onInsert={insert} flow={flow}/></div>
}
