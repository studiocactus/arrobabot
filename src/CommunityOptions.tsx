import {useEffect,useState} from 'react';
import {api,errorText} from './api';
import {Field} from './components';
export default function CommunityOptions({profileId,kind,notify}:{profileId:string;kind:string;notify:(s:string)=>void}){
 const [config,setConfig]=useState<Record<string,unknown>>({});
 useEffect(()=>{void api<Record<string,unknown>>('module.config.get',{profileId,key:kind}).then(v=>setConfig(v||{})).catch(e=>notify(errorText(e)))},[profileId,kind]);
 const number=(key:string,label:string,fallback:number,min:number,max:number)=><Field label={label}><input type="number" min={min} max={max} value={Number(config[key]??fallback)} onChange={e=>setConfig({...config,[key]:Number(e.target.value)})}/></Field>;
 return <details><summary>Avançado</summary><div className="module-settings">
 {kind==='points'&&<><Field label="Nome da moeda"><input maxLength={30} value={String(config.currency||'pontos')} onChange={e=>setConfig({...config,currency:e.target.value})}/></Field>{number('amount','Pontos por participação',5,0,10000)}{number('interval','Intervalo mínimo por pessoa (segundos)',60,10,86400)}{number('subscriberMultiplier','Multiplicador para assinantes',2,1,10)}<p className="help">A pontuação considera mensagens recebidas. Espectadores silenciosos não podem ser medidos por estas APIs.</p></>}
 {kind==='songs'&&<>{number('perUser','Pedidos por pessoa',2,1,20)}{number('limit','Limite total da fila',50,1,500)}<p className="help">Use !musicas para consultar os próximos pedidos. A primeira música representa a seleção atual; finalize-a no painel quando terminar.</p></>}
 {kind==='games'&&<><Field label="Trivia automática"><select value={config.autoTrivia?'on':'off'} onChange={e=>setConfig({...config,autoTrivia:e.target.value==='on'})}><option value="off">Desativada</option><option value="on">Publicar durante baixa atividade</option></select></Field>{number('idleSeconds','Silêncio mínimo no chat (segundos)',300,60,3600)}{number('intervalSeconds','Intervalo entre perguntas (segundos)',900,300,86400)}<Field label="Perguntas e respostas" hint="Uma linha por pergunta, separada da resposta por |. Cada acerto vale 50 pontos."><textarea rows={5} placeholder="Quanto é 2 + 2? | 4" value={String(config.questions||'')} onChange={e=>setConfig({...config,questions:e.target.value})}/></Field><p className="help">Publica no canal conectado. Cada pergunta fica aberta por cinco minutos.</p></>}
 {kind==='discord'&&<Field label="Convite do servidor (!discord)"><input placeholder="https://discord.gg/seu-convite" value={String(config.invite||'')} onChange={e=>setConfig({...config,invite:e.target.value})}/></Field>}
 <footer className="form-footer"><button onClick={()=>api('module.config',{profileId,key:kind,value:config}).then(()=>notify('Configuração salva.')).catch(e=>notify(errorText(e)))}>Salvar configuração avançada</button></footer>
 </div></details>
}
