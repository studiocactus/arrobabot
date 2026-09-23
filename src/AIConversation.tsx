import {useEffect,useRef,useState} from 'react';
import {api,desktop,errorText} from './api';
import {Field} from './components';
import {newAction,newFlow,type Flow} from './types';

export const banterInstruction='Responda em uma frase curta, com humor de resenha de live e uma provocação leve sobre a jogada. Reconheça o que a pessoa acabou de dizer e mantenha o mesmo assunto. Não invente partidas ou acontecimentos anteriores.';
export function conversationFlow(profileId:string,pattern:string,instruction:string):Flow {
 return {...newFlow(profileId),name:'Resenha do chat',trigger:{kind:'contains',pattern,permission:'everyone',cooldown:60,userCooldown:120},actions:[{...newAction('ai.generate'),text:instruction,target:'local.aiResponse'},{...newAction('chat'),text:'{{local.aiResponse}}'}]};
}
export function AIResponseTest({profileId,instruction}:{profileId:string;instruction:string}){
 const [message,setMessage]=useState('O Thenees hoje está amassando na play');
 const [recent,setRecent]=useState('');const [reply,setReply]=useState('');const [error,setError]=useState('');const [busy,setBusy]=useState(false);
 const revision=useRef(0);
 useEffect(()=>{revision.current++;setReply('');setError('');setBusy(false);return()=>{revision.current++}},[profileId,instruction,message,recent]);
 return <details><summary>Testar resposta contextual</summary><p className="help">Chama a IA configurada neste perfil. Pode consumir créditos do provedor. Não publica no chat nem registra memórias; usa apenas a conversa de teste abaixo e as memórias existentes.</p>
 <Field label="Mensagem da pessoa"><textarea value={message} onChange={e=>{setMessage(e.target.value);setReply('')}} maxLength={2000}/></Field>
 <Field label="Conversa anterior (opcional)" hint="Uma fala por linha. Ex.: Bia: esta semana ele errou todos os tiros."><textarea value={recent} onChange={e=>{setRecent(e.target.value);setReply('')}} maxLength={6000}/></Field>
 <button type="button" disabled={!desktop||busy||!message.trim()} onClick={async()=>{const current=++revision.current;setBusy(true);setError('');setReply('');try{const answer=await api<string>('ai.preview',{profileId,instruction,message,recent});if(current===revision.current)setReply(answer)}catch(e){if(current===revision.current)setError(errorText(e))}finally{if(current===revision.current)setBusy(false)}}}>{busy?'Gerando…':'Gerar resposta de teste'}</button>
 {!desktop&&<p className="help">Disponível no aplicativo desktop com a IA configurada.</p>}{reply&&<p className="ai-reply" aria-live="polite">{reply}</p>}{error&&<p role="alert">{error}</p>}</details>
}
export function ConversationForm({profileId,onSave}:{profileId:string;onSave:(f:Flow)=>Promise<void>}){
 const [pattern,setPattern]=useState('amassando');const [instruction,setInstruction]=useState(banterInstruction);const [busy,setBusy]=useState(false);const [error,setError]=useState('');
 return <form className="form-pad" onSubmit={async e=>{e.preventDefault();setBusy(true);try{await onSave(conversationFlow(profileId,pattern.trim(),instruction))}catch(err){setError(errorText(err))}finally{setBusy(false)}}}>
 <p>Conecte uma fala do chat à IA. O bot gera uma resposta nova para cada mensagem que combinar com o gatilho.</p>
 <Field label="Responder quando a mensagem contiver"><input required value={pattern} onChange={e=>setPattern(e.target.value)} placeholder="amassando"/></Field>
 <Field label="Como a IA deve responder"><textarea required rows={4} value={instruction} onChange={e=>setInstruction(e.target.value)}/></Field>
 <p className="help">Usa a personalidade salva em IA e Memória. Intervalo de 60 segundos entre respostas e 120 por pessoa. Você pode ajustar tudo no editor da automação.</p>
 <AIResponseTest profileId={profileId} instruction={instruction}/>{error&&<p role="alert">{error}</p>}
 <footer className="form-footer"><button className="primary" disabled={busy||!pattern.trim()||!instruction.trim()}>Salvar resenha do chat</button></footer></form>
}
