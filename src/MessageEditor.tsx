import {useRef} from 'react';
import {Field} from './components';
import {VariableHelp} from './Variables';
import {insertVariable,messageParts} from './variableCatalog';
import type {Flow} from './types';

export default function MessageEditor({profileId,label,value,onChange,flow,required=false}:{profileId:string;label:string;value:string;onChange:(v:string)=>void;flow?:Flow;required?:boolean}){
 const ref=useRef<HTMLTextAreaElement>(null);const selection=useRef<{start:number;end:number}|null>(null);
 function insert(token:string){const range=selection.current||{start:value.length,end:value.length};const next=insertVariable(value,token,range.start,range.end);onChange(next.text);selection.current={start:next.caret,end:next.caret};requestAnimationFrame(()=>{ref.current?.focus();ref.current?.setSelectionRange(next.caret,next.caret)})}
 const parts=messageParts(value);
 return <div className="message-editor"><Field label={label} hint="Posicione o cursor e escolha uma informação nos botões abaixo."><textarea ref={ref} required={required} rows={4} value={value} onSelect={e=>{selection.current={start:e.currentTarget.selectionStart,end:e.currentTarget.selectionEnd}}} onChange={e=>onChange(e.target.value)} placeholder="Escreva sua mensagem…"/></Field>
 {parts.some(p=>p.label)&&<div className="message-readable"><small>Sua mensagem com os nomes das informações</small><p>{parts.map((p,i)=>p.label?<span className="variable-chip" key={i} title={p.text}>{p.label}</span>:<span key={i}>{p.text}</span>)}</p></div>}
 <div className="variable-shortcuts" aria-label="Inserção rápida"><button type="button" onClick={()=>insert('{{user}}')}>+ Nome da pessoa</button><button type="button" onClick={()=>insert('{{channel}}')}>+ Nome do canal</button><button type="button" onClick={()=>insert('{{rawInput}}')}>+ Texto do pedido</button></div>
 <VariableHelp profileId={profileId} text={value} onInsert={insert} flow={flow}/></div>
}
