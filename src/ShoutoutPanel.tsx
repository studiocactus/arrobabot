import {useEffect,useState} from 'react';
import {Save,Play,Sparkles,MessageSquare,Megaphone} from 'lucide-react';
import {api} from './api';
import {saveFlow,useFlows} from './FlowTools';
import TestFlowDialog from './TestFlow';
import {Field,Toggle} from './components';
import {friendlyMessage} from './errors';
import {gankFlow,readGank,isGankCustomized,GANK_TEXT_DEFAULT,GANK_PROMPT_DEFAULT,type GankFields} from './gankFlow';
import type {Profile,Flow} from './types';

/** Shoutout e ganks: escolhe como a raid recebida é anunciada. Por dentro é
 *  uma automação de raid de verdade (mesmo motor), editável em Automações. */

const camposIniciais=():GankFields=>({shoutout:true,ai:false,announce:false,text:GANK_TEXT_DEFAULT,prompt:GANK_PROMPT_DEFAULT});

/** Liga/desliga o anúncio junto com o módulo: cria a automação na primeira
 *  vez e depois só liga ou desliga — nenhuma outra parte é tocada. */
export async function syncGank(profileId:string,enabled:boolean){
  const flows=await api<Flow[]>('flows',{profileId});
  const existente=flows.find(f=>f.fnId==='gank');
  if(!existente){if(enabled)await saveFlow({...gankFlow(profileId,camposIniciais()),enabled:true});return}
  if(existente.enabled!==enabled)await saveFlow({...existente,enabled});
}

/** Amostra legível da mensagem, com exemplos no lugar das variáveis. */
const amostra=(texto:string)=>texto.replace(/\{\{raider\.name\}\}/g,'RaidMan').replace(/\{\{raid\.viewers\}\}/g,'42').replace(/\{\{raiderLogin\}\}/g,'raidman').replace(/\{\{raidViewers\}\}/g,'42');

export default function ShoutoutPanel({profile,notify,onSaved}:{profile:Profile;notify:(s:string)=>void;onSaved:()=>void}){
 const {flows,refresh}=useFlows(profile.id,notify);
 const existente=flows.find(f=>f.fnId==='gank');
 const customized=!!existente&&isGankCustomized(existente);
 const outras=flows.filter(f=>f.enabled&&f.trigger.kind==='raid'&&f.fnId!=='gank');
 const [f,setF]=useState<GankFields>(camposIniciais);
 const [carregado,setCarregado]=useState('');
 const [testing,setTesting]=useState<Flow|null>(null);
 useEffect(()=>{if(existente&&existente.id!==carregado){setF(readGank(existente));setCarregado(existente.id)}},[existente?.id]);
 const on=!!profile.modules?.shoutout;
 const set=(p:Partial<GankFields>)=>setF({...f,...p});
 const pronto=()=>({...gankFlow(profile.id,f,existente??undefined),enabled:on});
 async function salvar(){
  try{const latest=await api<Flow[]>('flows',{profileId:profile.id});const saved=latest.find(flow=>flow.fnId==='gank');if(saved&&isGankCustomized(saved))throw Error('Este fluxo tem passos personalizados. Edite-o em Avançado → Automações para preservar os ajustes.');await saveFlow({...gankFlow(profile.id,f,saved),enabled:saved?.enabled??on});await refresh();onSaved();notify('Shoutout salvo.')}
  catch(e){notify(friendlyMessage(e))}
 }
 return <>
 <p className="help">Quando outra pessoa traz a raid (gank), o BotLive anuncia <strong>quem chegou</strong> — com destaque nativo da Twitch e/ou uma mensagem, fixa ou escrita pela IA naquele momento. Nada é publicado até você ativar este módulo.</p>
 {!on&&<div className="notice"><Megaphone size={16}/> O módulo está desativado: o anúncio não dispara na live. Ative pela chave acima.</div>}
 {!!outras.length&&<div className="notice">Você já tem outra automação de raid ativa (<strong>{outras.map(o=>o.name).join('", "')}</strong>). Este painel cuida da dele mesmo — confira as duas para a raid não ser respondida em dobro.</div>}
 {customized&&<div className="notice">Esta automação foi personalizada no editor avançado. Continue a edição em Avançado → Automações para preservar os passos extras.</div>}
 <div className="two-columns">
  <div>
   <h4>Como anunciar</h4>
   <Toggle label="Destaque de canal (shoutout nativo)" checked={f.shoutout} onChange={shoutout=>set({shoutout})}/>
   <p className="help">A Twitch mostra o canal de quem raidou para os seus espectadores. A conta do bot precisa ser moderadora — confira em Perfis.</p>
   <div className="row wrap">
    <button type="button" className={f.announce?'':'primary'} aria-pressed={f.announce} onClick={()=>set({announce:true})}>Anúncio da Twitch</button>
    <button type="button" className={!f.announce?'':'primary'} aria-pressed={!f.announce} onClick={()=>set({announce:false})}>Mensagem normal no chat</button>
   </div>
   <p className="help">O anúncio ganha destaque na Twitch; a mensagem normal aparece como qualquer fala no chat.</p>
  </div>
  <div>
   <h4>Que mensagem enviar</h4>
   <div className="row wrap">
    <button type="button" className={!f.ai?'':'primary'} aria-pressed={!f.ai} onClick={()=>set({ai:false})}><MessageSquare size={15}/>Mensagem fixa</button>
    <button type="button" className={f.ai?'':'primary'} aria-pressed={f.ai} onClick={()=>set({ai:true})}><Sparkles size={15}/>Aleatória por IA</button>
   </div>
   {f.ai
    ?<>
     <Field label="Instrução para a IA"><textarea rows={4} value={f.prompt} maxLength={900} onChange={e=>set({prompt:e.target.value})}/></Field>
     <Field label="Reserva (se a IA falhar)"><input value={f.text} maxLength={450} onChange={e=>set({text:e.target.value})}/></Field>
     <p className="help">A IA escreve na hora e nunca repete a anterior. Se o provedor falhar, a Reserva entra no lugar — a raid é agradecida de qualquer jeito.</p>
    </>
    :<>
     <Field label="Mensagem"><textarea rows={4} value={f.text} maxLength={450} onChange={e=>set({text:e.target.value})}/></Field>
     <p className="help">Use <code>{'{{raider.name}}'}</code> para quem mandou a raid e <code>{'{{raid.viewers}}'}</code> para quantos chegaram.</p>
    </>}
   <div className="notice" aria-label="Prévia do anúncio"><strong>Como pode sair:</strong> {f.ai?`A IA escreve na hora. Reserva: ${amostra(f.text)}`:`${f.shoutout?'Destaque de canal para quem raidou. ':''}${amostra(f.text)}`}</div>
  </div>
 </div>
 <footer className="form-footer"><span className="help">A automação aparece em Avançado → Automações para edição avançada e testes.</span><button type="button" onClick={()=>setTesting({...pronto()})}><Play size={15}/>Testar anúncio</button><button className="primary" disabled={customized} onClick={()=>void salvar()}><Save size={16}/>Salvar anúncio</button></footer>
 {testing&&<TestFlowDialog flow={testing} profileId={profile.id} onClose={()=>setTesting(null)}/>}
 </>;
}
