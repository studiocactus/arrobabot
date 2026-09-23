import {useState,useEffect} from 'react';
import {api,errorText} from './api';
import {Field} from './components';
export default function ObsidianBridge({profileId,notePath,notify}:{profileId:string;notePath?:string;notify:(s:string)=>void}){
 const [endpoint,setEndpoint]=useState('http://127.0.0.1:27123');const [key,setKey]=useState('');
 useEffect(()=>{api<{endpoint?:string}>('module.config.get',{profileId,key:'obsidian'}).then(v=>{if(v?.endpoint)setEndpoint(v.endpoint)}).catch(()=>{})},[profileId]);
 return <details><summary>Ponte opcional com Obsidian</summary><p className="help">Para quem já usa o plugin Local REST API. A nota selecionada é copiada para BotLive / ID do perfil dentro do seu vault Obsidian, preservando o isolamento. Não é necessário usar esta integração para ter memória no BotLive.</p><div className="form-grid"><Field label="Endereço local do plugin"><input value={endpoint} onChange={e=>setEndpoint(e.target.value)}/></Field><Field label="Chave de API do plugin"><input type="password" autoComplete="new-password" value={key} onChange={e=>setKey(e.target.value)} placeholder="Deixe vazio para manter a chave"/></Field></div><button disabled={!notePath} onClick={async()=>{try{await api('module.config',{profileId,key:'obsidian',value:{endpoint}});if(key){await api('secret.save',{profileId,key:'obsidian_key',value:key});setKey('')}const r=await api<{path:string}>('obsidian.sync',{profileId,path:notePath});notify('Nota copiada para '+r.path)}catch(e){notify(errorText(e))}}}>Copiar nota salva para o Obsidian</button></details>
}
