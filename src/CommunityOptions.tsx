import {useEffect,useState} from 'react';
import {api,errorText} from './api';
import {Field} from './components';
import {minutesToSeconds,secondsToMinutes} from './types';
export default function CommunityOptions({profileId,kind,notify}:{profileId:string;kind:string;notify:(s:string)=>void}){
 const [config,setConfig]=useState<Record<string,unknown>>({});
 useEffect(()=>{void api<Record<string,unknown>>('module.config.get',{profileId,key:kind}).then(v=>setConfig(v||{})).catch(e=>notify(errorText(e)))},[profileId,kind]);
 const number=(key:string,label:string,fallback:number,min:number,max:number)=><Field label={label}><input type="number" min={min} max={max} value={Number(config[key]??fallback)} onChange={e=>setConfig({...config,[key]:Number(e.target.value)})}/></Field>;
 const minutes=(key:string,label:string,fallbackSec:number,min:number,max:number)=><Field label={label}><input type="number" min={min} max={max} step="any" value={secondsToMinutes(Number(config[key]??fallbackSec))} onChange={e=>setConfig({...config,[key]:minutesToSeconds(Number(e.target.value)||0)})}/></Field>;
 return <details><summary>Avançado</summary><div className="module-settings">
 {kind==='points'&&<><Field label="Nome da moeda"><input maxLength={30} value={String(config.currency||'pontos')} onChange={e=>setConfig({...config,currency:e.target.value})}/></Field>{number('amount','Pontos por participação',5,0,10000)}{minutes('interval','Intervalo mínimo por pessoa (minutos)',60,0,1440)}{number('subscriberMultiplier','Multiplicador para assinantes',2,1,10)}<p className="help">A pontuação considera mensagens recebidas. Espectadores silenciosos não podem ser medidos por estas APIs.</p></>}
 {kind==='songs'&&<>{number('perUser','Pedidos por pessoa',2,1,20)}{number('limit','Limite total da fila',50,1,500)}<p className="help">Use !musicas para consultar os próximos pedidos. A primeira música representa a seleção atual; finalize-a no painel quando terminar.</p></>}
 {kind==='games'&&<><Field label="Trivia automática"><select value={config.autoTrivia?'on':'off'} onChange={e=>setConfig({...config,autoTrivia:e.target.value==='on'})}><option value="off">Desativada</option><option value="on">Publicar durante baixa atividade</option></select></Field>{minutes('idleSeconds','Silêncio mínimo no chat (minutos)',300,1,60)}{minutes('intervalSeconds','Intervalo entre perguntas (minutos)',900,5,1440)}<Field label="Perguntas e respostas" hint="Uma linha por pergunta, separada da resposta por |. Cada acerto vale 50 pontos."><textarea rows={5} placeholder="Quanto é 2 + 2? | 4" value={String(config.questions||'')} onChange={e=>setConfig({...config,questions:e.target.value})}/></Field><p className="help">Publica no canal conectado. Cada pergunta fica aberta por cinco minutos.</p></>}
 {kind==='affinity'&&<>{/* Padrões espelhados em modules.rs (AFFINITY_TERMS e AFFINITY_MESSAGES). */}
   <Field label="Palavras-gatilho" hint="Vírgula entre termos; só a palavra inteira dispara (amor sim, amortecedor não)."><input maxLength={300} value={String(config.terms??'amor, paixão, crush, casal, beijo, namorar')} onChange={e=>setConfig({...config,terms:e.target.value})}/></Field>
   <Field label="Comando do par" hint="!match nome1 nome2 compara dois nomes quaisquer com a mesma conta."><input maxLength={30} value={String(config.command||'!match')} onChange={e=>setConfig({...config,command:e.target.value})}/></Field>
   {minutes('cooldown','Intervalo entre matches (minutos)',120,0,1440)}
   {minutes('userCooldown','Intervalo por pessoa (minutos)',300,0,1440)}
   <Field label="A live como par"><select value={config.includeStreamer?'on':'off'} onChange={e=>setConfig({...config,includeStreamer:e.target.value==='on'})}><option value="off">Fora do sorteio</option><option value="on">Pode ser sorteada</option></select></Field>
   <Field label="Mensagens por faixa" hint="Uma linha por faixa, da menor para a maior: 0–20, 21–40, 41–60, 61–80 e 81–100%. Use {{user1}}, {{user2}} e {{pct}}. Linha vazia usa o texto padrão."><textarea rows={5} value={String(config.messages??'')} onChange={e=>setConfig({...config,messages:e.target.value})}/></Field>
   <p className="help">A % vem de um hash dos dois nomes: o mesmo casal dá sempre o mesmo resultado. O par é sorteado entre quem falou no chat nos últimos dez minutos, e a menção com @ usa a pessoa marcada quando ela está por ali.</p></>}
  {kind==='discord'&&<Field label="Convite do servidor (!discord)"><input placeholder="https://discord.gg/seu-convite" value={String(config.invite||'')} onChange={e=>setConfig({...config,invite:e.target.value})}/></Field>}
 <footer className="form-footer"><button onClick={()=>api('module.config',{profileId,key:kind,value:config}).then(()=>notify('Configuração salva.')).catch(e=>notify(errorText(e)))}>Salvar configuração avançada</button></footer>
 </div></details>
}
