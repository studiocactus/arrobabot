import {useEffect,useRef,useState} from 'react';
import type {CodeBlock} from './codeMode';

type CodeTab='fields'|'html'|'css'|'js'|'data';

const TABS:[CodeTab,string][]=[['fields','Fields'],['html','HTML'],['css','CSS'],['js','JS'],['data','Data']];

const HINTS:Record<Exclude<CodeTab,'fields'|'data'>,string>={
 html:'HTML extra no topo da camada — moldura, fundo, ornamento. Scripts aqui não executam; o que é código entra na aba JS.',
 css:'Vale só dentro da camada (no Estúdio não pinta o aplicativo): `.bl-chat-name{font-weight:400}`. Seletor de `:root`/`body` pinta a própria camada.',
 js:'Roda na prévia e no OBS com `root` = a camada. Erro de sintaxe aparece no console e não derruba o resto.',
};

/**
 * Abas Fields · HTML · CSS · JS · Data do modo código. `Fields` continua sendo
 * o formulário da própria tela (children); os outros quatro tabs editam o
 * código do streamer, e `Data` mostra a configuração inteira em JSON — o
 * mesmo caminho de salvamento do formulário, normalizado antes de aceitar.
 */
export function CodeTabs({
 code,onChange,dataJson,defaultData,onApplyData,children,
}:{
 code:CodeBlock;
 onChange:(c:CodeBlock)=>void;
 dataJson:string;
 defaultData:string;
 onApplyData:(json:string)=>string|null;
 children:React.ReactNode;
}){
 const [tab,setTab]=useState<CodeTab>('fields');
 const [draft,setDraft]=useState('');
 const [err,setErr]=useState('');
 const lastTab=useRef<CodeTab>('fields');
 // Entrou na aba Data (ou o JSON mudou por fora): rascunho espelha o atual.
 useEffect(()=>{
  if(tab==='data'&&lastTab.current!=='data')setDraft(dataJson);
  lastTab.current=tab;
  setErr('');
 },[tab,dataJson]);

 return <div className="cd-tabs">
  <div className="cd-tabbar" role="tablist" aria-label="Modo código">
   {TABS.map(([k,l])=><button key={k} type="button" role="tab" aria-selected={tab===k}
     className={'lp-seg'+(tab===k?' selected':'')} onClick={()=>setTab(k)}>{l}</button>)}
  </div>
  {tab==='fields'&&<div className="cd-fields">{children}</div>}
  {tab!=='fields'&&tab!=='data'&&<div className="cd-panel">
   <p className="lp-note">{HINTS[tab]}</p>
   <textarea className="cd-code" spellCheck={false} rows={16} aria-label={`Código ${tab}`}
    value={code[tab]} onChange={e=>onChange({...code,[tab]:e.target.value})}/>
   <div className="cd-actions">
    <button type="button" onClick={()=>onChange({...code,[tab]:''})}>Restaurar padrão</button>
    <small className="lp-note">{code[tab].length} caracteres · salva sozinho</small>
   </div>
  </div>}
  {tab==='data'&&<div className="cd-panel">
   <p className="lp-note">A configuração desta tela em JSON — o mesmo que os botões do formulário gerenciam. <strong>Aplicar</strong> normaliza como o formulário faria e recarrega a tela.</p>
   <textarea className="cd-code" spellCheck={false} rows={16} aria-label="Dados da configuração (JSON)"
    value={draft} onChange={e=>setDraft(e.target.value)}/>
   {err&&<p role="alert" className="inline-error">{err}</p>}
   <div className="cd-actions">
    <button type="button" className="primary" onClick={()=>{const e=onApplyData(draft);setErr(e||'');if(!e)setTab('fields')}}>Aplicar</button>
    <button type="button" onClick={()=>setDraft(defaultData)}>Restaurar padrão</button>
   </div>
  </div>}
 </div>;
}
