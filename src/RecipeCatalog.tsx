import {useState} from 'react';
import {Field,Modal} from './components';
import {recipes,recipeDraft,recipeConfiguration,type RecipeId} from './recipes';
import type {Flow} from './types';
import {errorText} from './api';

export default function RecipeCatalog({profileId,flows,onPick,onClose}:{profileId:string;flows:Flow[];onPick:(id:RecipeId)=>Promise<void>;onClose:()=>void}){
 const [selected,setSelected]=useState<RecipeId>('welcome');
 const [query,setQuery]=useState('');
 const [busy,setBusy]=useState(false);
 const [error,setError]=useState('');
 const recipe=recipes.find(item=>item.id===selected)!;
 const draft=recipeDraft(selected,profileId);
 const existing=recipeConfiguration(selected,profileId,flows);
 const text=(value:string)=>value.normalize('NFD').replace(/[̀-ͯ]/g,'').toLowerCase();
 const visible=recipes.filter(item=>text(query).trim().split(/\s+/).every(word=>text(item.title+' '+item.group+' '+item.result).includes(word)));
 const showSelected=visible.some(item=>item.id===selected);
 return <Modal wide title="Receitas prontas" onClose={()=>{if(!busy)onClose()}}>
  <div className="form-pad recipe-catalog">
   <p className="help">Escolha um resultado e revise a configuração antes de salvar. Escolher uma receita não ativa nem executa nada.</p>
   <Field label="Buscar receita"><input value={query} onChange={event=>setQuery(event.target.value)} placeholder="Ex.: voz, raid, água ou seguidores"/></Field>
   <div className="recipe-grid" role="group" aria-label="Receitas disponíveis">
    {visible.map(item=><button type="button" key={item.id} aria-label={item.title} aria-pressed={selected===item.id} onClick={()=>{setSelected(item.id);setError('')}} disabled={busy}><strong>{item.title}</strong><span>{item.group}</span></button>)}
   </div>
   {!visible.length&&<p role="status" className="help">Nenhuma receita encontrada. Tente outro termo.</p>}
   {!showSelected&&visible.length>0&&<p role="status" className="help">Selecione uma receita para conferir o resultado.</p>}
   {showSelected&&<section className="recipe-summary" aria-label="Resumo da receita">
    <h3>{recipe.title}</h3>
    <p><strong>Quando acontece:</strong> {existing?(existing.trigger.kind==='command'?'Comando salvo: '+existing.trigger.pattern:'Conforme o gatilho da configuração já salva.'):recipe.when}</p>
    <p><strong>Resultado:</strong> {existing?'Abre seus ajustes atuais, sem reaplicar o modelo.':recipe.result}</p>
    <p><strong>O que precisa:</strong> {recipe.requirements}</p>
    <p className="help">{existing?('Já existe “'+existing.name+'”. Você vai editar essa configuração, preservando seus ajustes.'):(draft||selected==='conversation'?'A configuração nova começa desativada. Escolher a receita não salva nem executa nada.':'Você vai abrir o painel que já cuida dessa função, com suas configurações atuais.')}</p>
   </section>}
   {error&&<p className="inline-error" role="alert">{error}</p>}
   <footer className="form-footer"><span className="help">Para importar ou compartilhar seus próprios pacotes, use Biblioteca de presets.</span><button type="button" className="primary" disabled={busy||!showSelected} onClick={async()=>{setBusy(true);setError('');try{await onPick(selected)}catch(err){setError(errorText(err))}finally{setBusy(false)}}}>{busy?'Abrindo…':existing?'Editar configuração existente':'Configurar receita'}</button></footer>
  </div>
 </Modal>;
}
