import {useEffect,useState} from 'react';
import {open} from '@tauri-apps/plugin-dialog';
import {FolderOpen,RefreshCw,Trash2} from 'lucide-react';
import {api,errorText} from './api';
import {Card,Field,Toggle,Tag} from './components';
import {aiAnchors,aiLengths,knowledgeDepths,type AIConfig,type KnowledgeEntry} from './types';
import {knowledgeGroups,toggleKnowledgeFile} from './aiOptions';

/** Base de conhecimento: importar a pasta, escolher nicho/profundidade e ligar/desligar arquivos. */
export function KnowledgeCard({profileId,ai,onChange,notify}:{profileId:string;ai:AIConfig;onChange:(ai:AIConfig)=>void;notify:(s:string)=>void}){
 const [items,setItems]=useState<KnowledgeEntry[]>([]);
 const refresh=()=>api<KnowledgeEntry[]>('knowledge.list',{profileId}).then(setItems).catch(()=>setItems([]));
 useEffect(()=>{setItems([]);void refresh()},[profileId]);
 async function importFrom(source:string){
  try{
   const made=await api<{files:number}>('knowledge.import',{profileId,path:source});
   onChange({...ai,knowledgeSource:source,knowledge:true,knowledgeOff:[]});
   await refresh();
   notify(`${made.files} arquivos na base. A pasta original pode ser movida depois sem quebrar nada.`);
  }catch(e){notify(errorText(e))}
 }
 async function chooseFolder(){
  try{
   const dir=await open({directory:true,multiple:false,title:'Escolha a pasta da base de conhecimento'});
   if(typeof dir!=='string')return;
   await importFrom(dir);
  }catch(e){notify(errorText(e))}
 }
 async function removeFile(path:string){
  try{
   await api('knowledge.remove',{profileId,path});
   onChange({...ai,knowledgeOff:(ai.knowledgeOff||[]).filter(p=>p!==path)});
   await refresh();
   notify('Arquivo removido da base.');
  }catch(e){notify(errorText(e))}
 }
 const groups=knowledgeGroups(items,ai);
 const niches=[...new Set(items.filter(i=>i.category==='nichos').map(i=>i.file))];
 return <Card title="Base de conhecimento" action={<Tag color={ai.knowledge?'green':'gray'}>{items.length} arquivos</Tag>}>
  <p className="help">Importe a pasta <strong>documentação\knowledge</strong>. Ela é copiada para os dados do perfil e a IA lê os arquivos antes de responder: mover ou apagar a pasta original depois não quebra nada.</p>
  <div className="switch-row"><div><strong>Usar a base nas respostas</strong><small>Desligue para responder só com a personalidade e as memórias.</small></div><Toggle label="Usar a base de conhecimento" checked={ai.knowledge} onChange={knowledge=>onChange({...ai,knowledge})}/></div>
  <div className="form-grid">
   <Field label="Nicho do canal" hint="A pasta de nichos escolhida entra na frente."><input list="knowledge-niches" value={ai.knowledgeNicho} placeholder="Ex.: fps" onChange={e=>onChange({...ai,knowledgeNicho:e.target.value})}/><datalist id="knowledge-niches">{niches.map(n=><option key={n} value={n}/>)}</datalist></Field>
   <Field label="Profundidade" hint="Quanto texto cabe em cada resposta."><select value={ai.knowledgeDepth} onChange={e=>onChange({...ai,knowledgeDepth:e.target.value})}>{Object.entries(knowledgeDepths).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>
  </div>
  <div className="list-tools">
   <button className="primary" onClick={chooseFolder}><FolderOpen size={15}/>Importar pasta</button>
   {ai.knowledgeSource&&<button onClick={()=>importFrom(ai.knowledgeSource)}><RefreshCw size={15}/>Atualizar da pasta original</button>}
   {ai.knowledgeSource&&<span className="list-count" title={ai.knowledgeSource}>{ai.knowledgeSource}</span>}
  </div>
  {groups.length?groups.map(g=><div key={g.category}><div className="list-count">{g.category}</div>{g.files.map(({entry,included})=><div className="list-row" key={entry.path}><div><strong>{entry.title||entry.file}</strong><small>{entry.path}{entry.size?` · ${Math.max(1,Math.round(entry.size/1024))} kB`:''}</small></div><div className="row"><button className="icon-button danger" aria-label={`Remover ${entry.path}`} onClick={()=>removeFile(entry.path)}><Trash2 size={15}/></button><Toggle label={`Usar ${entry.path}`} checked={included} onChange={()=>onChange(toggleKnowledgeFile(ai,entry.path))}/></div></div>)}</div>):<p className="help">Nenhum arquivo ainda. Importe a pasta da documentação para começar.</p>}
 </Card>;
}

/** Padrões do perfil: o que toda ação nova de IA herda. */
export function ResponseDefaults({ai,onChange}:{ai:AIConfig;onChange:(ai:AIConfig)=>void}){
 return <Card title="Como responder" action={<Tag color="blue">Padrões</Tag>}>
  <p className="help">Vale para todo bloco novo de IA. Cada ação pode mudar na hora de editar o comando.</p>
  <div className="form-grid">
   <Field label="Onde a resposta se ancora"><select value={ai.anchor||'all'} onChange={e=>onChange({...ai,anchor:e.target.value})}>{Object.entries(aiAnchors).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>
   <Field label="Tamanho da resposta"><select value={ai.answerLength} onChange={e=>onChange({...ai,answerLength:e.target.value})}><option value="">Automático: até 120 caracteres</option>{Object.entries(aiLengths).map(([k,n])=><option key={k} value={k}>{n}</option>)}</select></Field>
  </div>
  <div className="switch-row"><div><strong>Evitar repetição</strong><small>Orienta a IA a variar em relação às últimas respostas do bot.</small></div><Toggle label="Evitar repetição" checked={ai.noRepeat} onChange={noRepeat=>onChange({...ai,noRepeat})}/></div>
 </Card>;
}
