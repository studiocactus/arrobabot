import {useState} from 'react';
import {Plane,ArrowRight,CheckCircle2,RotateCw} from 'lucide-react';
import {api} from './api';
import {friendlyMessage} from './errors';
import {Card} from './components';

type Item={message:string;page:string};

/** Pré-voo da live: varredura estática das configurações que só quebrariam ao vivo
 * (alertas e chat desligados, sons apagados, ações sem módulo ou autorização).
 * Sem rede e sem alterar nada: o relatório aponta o problema e leva à tela. */
export default function PreflightCard({profileId,onGo,labelOf}:{profileId:string;onGo:(id:string)=>void;labelOf:(id:string)=>string}){
 const [items,setItems]=useState<Item[]|null>(null);
 const [error,setError]=useState('');
 const [checking,setChecking]=useState(false);
 async function check(){
  if(checking)return;
  setChecking(true);setError('');
  try{const r=await api<{items:Item[]}>('preflight.check',{profileId});setItems(r.items)}
  catch(e){setError(friendlyMessage(e))}
  finally{setChecking(false)}
 }
 const body=items===null
  ? <p className="help">Antes de ir ao ar, o pré-voo confere o que só quebraria na live: alertas e chat desligados, sons apagados e ações sem módulo ou autorização. Nada é alterado.</p>
  : items.length===0
   ? <p className="preflight-ok"><CheckCircle2 size={17}/> Tudo pronto para a live: nada encontrado no perfil selecionado.</p>
   : <ul className="preflight-list">{items.map((it,i)=><li key={i}><span>{it.message}</span><button type="button" onClick={()=>onGo(it.page)}>Ir para {labelOf(it.page)}<ArrowRight size={14}/></button></li>)}</ul>;
 return <Card title="Pré-voo da live">
  {error&&<p role="alert" className="inline-error">{error}</p>}
  {body}
  <div className="card-bottom">
   <button type="button" disabled={checking} onClick={()=>{void check()}}>{checking?<><RotateCw className="spin" size={16}/>Verificando…</>:items===null?<><Plane size={16}/>Rodar pré-voo</>:<><RotateCw size={16}/>Verificar de novo</>}</button>
  </div>
 </Card>;
}
