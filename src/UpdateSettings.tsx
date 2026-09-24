import {useState} from 'react';
import {version} from '../package.json';
import {api,desktop,errorText} from './api';
import {Card,Field,Toggle} from './components';
type Result={available:boolean;version?:string;notes?:string};
export default function UpdateSettings({settings,setSettings,persist,loaded}:{settings:Record<string,unknown>;setSettings:(s:Record<string,unknown>)=>void;persist:(key:string,value:unknown)=>Promise<void>;loaded:boolean}){
 const [busy,setBusy]=useState('');const [result,setResult]=useState<Result|null>(null);const [status,setStatus]=useState('');const [error,setError]=useState('');
 async function run(install:boolean){
  setBusy(install?'install':'check');setError('');setStatus(install?'Baixando e verificando a assinatura. Aguarde o instalador…':'Consultando o canal de atualizações…');
  try{
   if(!install){setResult(null);await persist('updateEndpoint',String(settings.updateEndpoint||''));await persist('updatePublicKey',String(settings.updatePublicKey||''));}
   const response=await api<Result>(install?'update.install':'update.check');setResult(response);
   setStatus(response.available?(install?'Atualização entregue ao instalador. Conclua a instalação e abra o BotLive novamente.':'Versão '+response.version+' disponível. Confira as notas e clique em Instalar atualização.'):'Você está na versão mais recente disponível neste canal.');
  }catch(e){setStatus('');setError(errorText(e));}finally{setBusy('');}
 }
 return <Card title="Atualizações"><p>Versão instalada: <strong>{version}</strong></p><p className="help">Verifique e instale por aqui, sem abrir o GitHub. O aplicativo confirma a assinatura antes de instalar. Salve seu trabalho antes da instalação: ela pode fechar o BotLive.</p>
 {!desktop&&<p className="notice">Atualizações estão disponíveis no aplicativo instalado, não na prévia do navegador.</p>}
 <fieldset disabled={!loaded||!!busy} style={{border:0,padding:0,minWidth:0}}><details><summary>Configuração avançada do canal</summary><Field label="Endereço HTTPS do manifesto"><input value={String(settings.updateEndpoint||'')} onChange={e=>{setResult(null);setSettings({...settings,updateEndpoint:e.target.value})}}/></Field><Field label="Chave pública do atualizador"><textarea rows={3} value={String(settings.updatePublicKey||'')} onChange={e=>{setResult(null);setSettings({...settings,updatePublicKey:e.target.value})}}/></Field></details><div className="switch-row"><span>Verificar ao abrir</span><Toggle label="Verificar atualizações ao abrir" checked={!!settings.autoUpdate} onChange={v=>void persist('autoUpdate',v).catch(e=>setError(errorText(e)))}/></div>
 <button disabled={!desktop} onClick={()=>void run(false)}>{busy==='check'?'Verificando…':'Verificar atualização'}</button><button disabled={!desktop||!result?.available} onClick={()=>void run(true)}>{busy==='install'?'Instalando…':'Instalar atualização'}</button></fieldset>
 <p role="status" aria-live="polite">{!loaded?'Carregando configurações…':status}</p>{error&&<p role="alert" className="inline-error">{error}</p>}{result?.available&&result.notes&&<details open><summary>O que mudou na versão {result.version}</summary><pre style={{whiteSpace:'pre-wrap',fontFamily:'inherit'}}>{result.notes}</pre></details>}
 </Card>
}
