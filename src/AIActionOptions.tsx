import {Field} from './components';
import {aiAnchors,aiLengths,type Action,type AIConfig} from './types';
import {resolveAnchor,resolveLength,resolveKnowledge,resolveNoRepeat} from './aiOptions';

/** Controles por ação: o bloco pode sobrescrever o padrão do perfil ou herdar com valor vazio. */
export default function AIActionOptions({action,ai,onChange}:{action:Action;ai:AIConfig;onChange:(a:Action)=>void}){
 if(!action.kind.startsWith('ai'))return null;
 const anchor=resolveAnchor(action,ai);
 const length=resolveLength(action,ai);
 const knowledge=resolveKnowledge(action,ai);
 const noRepeat=resolveNoRepeat(action,ai);
 const triState=action.aiNoRepeat===null||action.aiNoRepeat===undefined?'':String(action.aiNoRepeat);
 return <details className="ai-options"><summary>Como esta ação responde</summary>
  <p className="help">Hoje vale: {aiAnchors[anchor]||aiAnchors.all} · {length?aiLengths[length]||'tamanho fixo':'até 120 caracteres'} · base {knowledge?'ligada':'desligada'} · {noRepeat?'sem repetição':'podendo repetir'}.</p>
  <Field label="Onde a resposta se ancora"><select value={action.aiAnchor||''} onChange={e=>onChange({...action,aiAnchor:e.target.value})}><option value="">Padrão do perfil</option>{Object.entries(aiAnchors).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>
  <Field label="Tamanho"><select value={action.aiLength||''} onChange={e=>onChange({...action,aiLength:e.target.value})}><option value="">Padrão do perfil</option>{Object.entries(aiLengths).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>
  <Field label="Base de conhecimento"><select value={action.aiKnowledge||''} onChange={e=>onChange({...action,aiKnowledge:e.target.value})}><option value="">Padrão do perfil</option><option value="on">Usar</option><option value="off">Não usar</option></select></Field>
  <Field label="Evitar repetição"><select value={triState} onChange={e=>onChange({...action,aiNoRepeat:e.target.value===''?null:e.target.value==='true'})}><option value="">Padrão do perfil</option><option value="true">Evitar repetição</option><option value="false">Pode repetir</option></select></Field>
  <Field label="Tom deste bloco" hint="Entra depois da personalidade, só nesta ação."><textarea rows={2} maxLength={600} value={action.aiStyle||''} onChange={e=>onChange({...action,aiStyle:e.target.value})} placeholder="Ex.: gírias do canal, tom de resenha"/></Field>
 </details>;
}
