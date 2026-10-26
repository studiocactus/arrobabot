import {useState} from 'react';
import {Field,Toggle} from './components';
import {FOLLOW_ASK_DEFAULT,FOLLOW_COUNT_DEFAULT,FOLLOW_SAY_OK_DEFAULT,FOLLOW_SAY_WARN_DEFAULT,FOLLOW_SONG_DEFAULT,clippersFlow,donationFlow,followAgeFlow,followersCountFlow,nowPlayingFlow} from './commandModels';
import type {Flow} from './types';

/** Formato de cada função: é o que permite editar pelo formulário sem abrir o
 *  grafo. A identidade vem de fnId/fnVersion no fluxo — nunca do nome do
 *  comando ou dos textos, que o usuário pode editar. */
const FORMATS:Record<string,string[][]>={
 'tempo-de-follow':[['followAge','condition','chat','condition','chat'],['followAge','ai.generate','chat']],
 'seguidores':[['chat']],
 'musica':[['chat']],
 'doacao':[['ai.generate','chat']],
 'clipadores':[['twitch','variable.increment','chat']],
};

export const FN_LABELS:Record<string,string>={'tempo-de-follow':'Tempo de follow','seguidores':'Total de seguidores','musica':'Música atual','doacao':'Link de doação','clipadores':'Clipadores'};

/** O fluxo ainda tem o formato da função? Se foi personalizado no editor
 *  avançado, o formulário não representa mais tudo — e nada é descartado. */
/** Qual função este formato representa? Baseado só no formato das etapas —
 *  nome e textos nunca decidem. Serve para adotar comandos antigos no
 *  formulário certo, com confirmação da pessoa. */
export function guessFn(flow:Flow):string|null{
 if(flow.trigger.kind!=='command'||flow.fnId)return null;
 const kinds=flow.actions.map(a=>a.kind);
 const matches=Object.entries(FORMATS).filter(([,formats])=>formats.some(f=>f.length===kinds.length&&f.every((k,i)=>k===kinds[i])));
 // Uma mensagem simples serve a várias funções. Não sugira uma conversão
 // arbitrária; timers e formatos ambíguos continuam em seus próprios editores.
 return matches.length===1?matches[0][0]:null;
}

export function templateMatches(flow:Flow):boolean{
 const formats=FORMATS[flow.fnId||''];
 if(!formats)return false;
 const kinds=flow.actions.map(a=>a.kind);
 return formats.some(f=>f.length===kinds.length&&f.every((k,i)=>k===kinds[i]));
}

type Fields={command:string;alias:string;mention:boolean;ai:boolean;ok:string;sayOk:string;sayWarn:string;link:string};

function read(flow:Flow):Fields{
 const pats=flow.trigger.pattern.split(',').map(s=>s.trim()).filter(Boolean);
 const a=flow.actions;
 const ai=a.some(x=>x.kind==='ai.generate');
 const sayOk=a.find(x=>x.kind==='chat')?.text||'';
 return {
  command:pats[0]||'',
  alias:pats.slice(1).join(', '),
  mention:a[0]?.target==='mention',
  ai,
  ok:a.find(x=>x.kind==='ai.generate')?.text||FOLLOW_ASK_DEFAULT,
  sayOk:ai?'':(sayOk||FOLLOW_SAY_OK_DEFAULT),
  sayWarn:a.filter(x=>x.kind==='chat')[1]?.text||FOLLOW_SAY_WARN_DEFAULT,
  link:(sayOk.split(' ').find(w=>w.includes('.')&&!w.includes('{{'))||''),
 };
}

/** Editor da função: as decisões da pessoa (essencial), o jeito de falar
 *  (personalizar) e o resto em avançado — sem variáveis, sem grafo. */
export function FunctionForm({flow,onChange,onAdvanced}:{flow:Flow;onChange:(f:Flow)=>void;onAdvanced:()=>void}){
 const [f,setF]=useState<Fields>(()=>read(flow));
 const fn=flow.fnId||'';
 const patch=(next:Partial<Fields>)=>{
  const v={...f,...next};setF(v);
  const fresh=fn==='tempo-de-follow'?followAgeFlow(flow.profileId,{command:v.command,alias:v.alias,mention:v.mention,ai:v.ai,ok:v.ok,sayOk:v.sayOk,sayWarn:v.sayWarn})
   :fn==='seguidores'?followersCountFlow(flow.profileId,{command:v.command,text:v.sayOk})
   :fn==='musica'?nowPlayingFlow(flow.profileId,{command:v.command,text:v.sayOk})
   :fn==='doacao'?donationFlow(flow.profileId,{command:v.command,link:v.link})
   :clippersFlow(flow.profileId,{command:v.command});
  // Identidade, ativação e ajustes avançados do gatilho ficam intactos.
  onChange({...flow,name:fresh.name,trigger:{...flow.trigger,kind:'command',pattern:fresh.trigger.pattern},actions:fresh.actions});
 };
 return <div className="form-pad">
  <p className="help"><strong>{FN_LABELS[fn]||'Função'}</strong> — preencha o essencial; o resto fica em opções.</p>
  <div className="form-grid">
   <Field label="Comando" hint="Aceita variações separadas por vírgula."><input value={f.command} onChange={e=>patch({command:e.target.value})}/></Field>
   {fn==='tempo-de-follow'&&<Field label="Alias (opcional)"><input value={f.alias} onChange={e=>patch({alias:e.target.value})}/></Field>}
   {fn==='doacao'&&<Field label="Seu link de doação" hint="Ex.: livepix.gg/seucanal."><input value={f.link} onChange={e=>patch({link:e.target.value})} placeholder="livepix.gg/seucanal"/></Field>}
  </div>
  {fn==='tempo-de-follow'&&<div className="switch-row"><div><strong>Consultar também @nome citado</strong><small>Com @usuario no chat, a consulta é da pessoa citada; sem ele, é de quem enviou.</small></div><Toggle label="Consultar também @nome citado" checked={f.mention} onChange={v=>patch({mention:v})}/></div>}
  {fn!=='doacao'&&fn!=='clipadores'&&<div className="switch-row"><div><strong>Escrever com IA no meu jeito</strong><small>Desligado, a resposta sai pronta e determinística com os dados reais. Ligado, a IA escreve a frase — e se ela falhar, a resposta pronta entra no lugar.</small></div><Toggle label="Escrever com IA no meu jeito" checked={f.ai} onChange={v=>patch({ai:v})}/></div>}
  {fn==='tempo-de-follow'&&!f.ai&&<><Field label="Resposta" hint="Use os campos legíveis: nome do usuário, canal, tempo seguindo, data do follow."><textarea rows={2} value={f.sayOk} onChange={e=>patch({sayOk:e.target.value})}/></Field><Field label="Resposta quando não der" hint="Quem não segue, não foi encontrado ou quando a consulta falha — a falha nunca vira “não segue”."><textarea rows={2} value={f.sayWarn} onChange={e=>patch({sayWarn:e.target.value})}/></Field></>}
  {fn==='tempo-de-follow'&&f.ai&&<Field label="Como o ArrobaSrv deve responder" hint="A frase é da IA; os dados entram exatamente como a Twitch informou e as falhas viram aviso fiel."><textarea rows={4} value={f.ok} onChange={e=>patch({ok:e.target.value})}/></Field>}
  {fn==='tempo-de-follow'&&f.ai&&<Field label="Resposta de reserva (se a IA falhar)" hint="Vai ao chat no lugar de deixar o comando mudo — com os dados reais, sem inventar nada."><textarea rows={2} value={f.sayOk||FOLLOW_SAY_OK_DEFAULT} onChange={e=>patch({sayOk:e.target.value})}/></Field>}
  {(fn==='seguidores'||fn==='musica')&&<Field label="Resposta"><textarea rows={2} value={f.sayOk} onChange={e=>patch({sayOk:e.target.value})}/></Field>}
  <p className="help"><strong>O que vai acontecer:</strong> quem digitar {f.command||'o comando'} recebe {fn==='tempo-de-follow'?(f.ai?'uma frase da IA com o tempo de follow (e a resposta pronta se a IA falhar)':'a resposta pronta com o tempo de follow'):fn==='seguidores'?'o total de seguidores':fn==='musica'?'a música tocando agora':fn==='doacao'?'uma frase da IA com seu link':'o clipe criado e a contagem de clipe'} — tudo no chat.</p>
  <p className="help">Permissões, intervalos e etapas extras ficam em <button type="button" onClick={onAdvanced}>Personalizar no editor avançado</button>.</p>
 </div>;
}
