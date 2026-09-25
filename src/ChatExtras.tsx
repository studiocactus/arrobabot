import AudioOutput from './AudioOutput';
import {useEffect,useState} from 'react';
import {open} from '@tauri-apps/plugin-dialog';
import {api,desktop,errorText} from './api';
import {AccordionItem,Card,Field,ItemStack,ListTools,Toggle} from './components';
import {playViewerSound} from './viewerSound';
type Asset={id:string;kind:string;name:string};
type Reply={id:string;enabled:boolean;keyword:string;matching:string;selection:string;asset:string;cooldown:number;userCooldown:number};
type Person={trigger:string;id:string;enabled:boolean;name:string;nickname:string;userId:string;asset:string;mode:string;cooldown:number;volume:number};
type Config={presenceEnabled:boolean;audioDeviceId:string;repliesEnabled:boolean;soundsEnabled:boolean;replies:Reply[];people:Person[]};
const blank:Config={presenceEnabled:false,audioDeviceId:'',repliesEnabled:false,soundsEnabled:false,replies:[],people:[]};
const MATCH:Record<string,string>={word:'Palavra inteira',contains:'Contém trecho'};
const SELECT:Record<string,string>={random:'Aleatória',sequence:'Sequência'};
const TRIGGER:Record<string,string>={message:'Ao falar',join:'Ao entrar',either:'Entrar ou falar'};
const MODE:Record<string,string>={first:'Uma vez por sessão',interval:'Com intervalo'};
export default function ChatExtras({profileId}:{profileId:string}){
 const [config,setConfig]=useState<Config>(blank);const [assets,setAssets]=useState<Asset[]>([]);const [error,setError]=useState('');const [message,setMessage]=useState('');const [busy,setBusy]=useState(false);const [loaded,setLoaded]=useState(!desktop);
 const [preview,setPreview]=useState<{count:number;lines:string[]}|null>(null);
 const [openIds,setOpenIds]=useState<Record<string,boolean>>({});
 const [qReplies,setQReplies]=useState('');const [qPeople,setQPeople]=useState('');
 useEffect(()=>{let live=true;if(desktop)api<{config:Config;assets:Asset[]}>('chatExtras.get',{profileId}).then(v=>{if(live){setConfig({...blank,...v.config,people:v.config.people.map(p=>({...p,trigger:p.trigger||'message'}))});setAssets(v.assets);setLoaded(true)}}).catch(e=>{if(live)setError(errorText(e))});return()=>{live=false}},[profileId]);
 async function run(fn:()=>Promise<void>){setBusy(true);setError('');setMessage('');try{await fn()}catch(e){setError(errorText(e))}finally{setBusy(false)}}
 async function choose(kind:string,update:(asset:string)=>void){await run(async()=>{
  const path=await open({multiple:false,filters:[{name:kind==='txt'?'Respostas UTF-8':'Áudio',extensions:kind==='txt'?['txt']:['wav','mp3','ogg']}]});if(typeof path!=='string')return;
  const asset=await api<Asset>('chatExtras.import',{profileId,kind,path});setAssets(a=>a.concat(asset));update(asset.id);
 })}
 const reply=(id:string,patch:Partial<Reply>)=>setConfig(c=>({...c,replies:c.replies.map(r=>r.id===id?{...r,...patch}:r)}));
 const person=(id:string,patch:Partial<Person>)=>setConfig(c=>({...c,people:c.people.map(r=>r.id===id?{...r,...patch}:r)}));
 const assetName=(id:string)=>assets.find(a=>a.id===id)?.name||'';
 const isOpen=(id:string)=>!!openIds[id];
 const flip=(id:string)=>setOpenIds(o=>({...o,[id]:!o[id]}));
 const setAll=(ids:string[],value:boolean)=>setOpenIds(o=>{const next={...o};ids.forEach(id=>next[id]=value);return next});
 const withItem=(id:string)=>setOpenIds(o=>({...o,[id]:true}));
 const addReply=()=>{const id=crypto.randomUUID();withItem(id);setConfig(c=>({...c,replies:c.replies.concat({id,enabled:true,keyword:'',matching:'word',selection:'random',asset:'',cooldown:30,userCooldown:60})}))};
 const addPerson=()=>{const id=crypto.randomUUID();withItem(id);setConfig(c=>({...c,people:c.people.concat({id,enabled:true,trigger:'message',name:'',nickname:'',userId:'',asset:'',mode:'first',cooldown:60,volume:0.7})}))};
 const file=(kind:string,value:string,change:(asset:string)=>void)=><div className="row"><select aria-label={kind==='txt'?'Arquivo de respostas':'Arquivo de som'} value={value} onChange={e=>change(e.target.value)}><option value="">Escolha um arquivo</option>{assets.filter(a=>a.kind===kind).map(a=><option key={a.id} value={a.id}>{a.name}</option>)}</select><button disabled={!desktop||busy} onClick={()=>void choose(kind,change)}>{kind==='txt'?'Vincular TXT':'Escolher som'}</button></div>;
 const replyRows=config.replies.map((r,i)=>({r,i}));
 const replyVisible=replyRows.filter(({r,i})=>!qReplies||`${r.keyword} ${assetName(r.asset)} regra ${i+1}`.toLowerCase().includes(qReplies.toLowerCase()));
 const personRows=config.people.map((r,i)=>({r,i}));
 const personVisible=personRows.filter(({r,i})=>!qPeople||`${r.nickname} ${r.name} ${assetName(r.asset)} pessoa ${i+1}`.toLowerCase().includes(qPeople.toLowerCase()));
 const allReplyIds=replyVisible.map(({r})=>r.id);const allPersonIds=personVisible.map(({r})=>r.id);
 return <div className="form-pad"><p>Respostas prontas para palavras do chat e uma trilha de chegada para cada pessoa. As alterações entram em vigor ao clicar em Salvar respostas e sons.</p>{!desktop&&<p className="notice">Prévia da configuração. Arquivos, reprodução e salvamento estão disponíveis no aplicativo desktop.</p>}
 <fieldset disabled={busy||!loaded} style={{border:0,padding:0,minWidth:0}}>
 <Card title="Respostas por arquivo TXT"><Toggle label="Ativar respostas TXT" checked={config.repliesEnabled} onChange={repliesEnabled=>setConfig({...config,repliesEnabled})}/><p className="help">Uma resposta por linha, até 450 caracteres. Linhas vazias são ignoradas. Use UTF-8. Alterações no arquivo original valem no próximo disparo. A primeira regra disponível responde; outras automações não executam nessa mensagem. Clique no título da regra para abrir ou fechar o formulário.</p>
 {config.replies.length===0
  ?<p className="help">Nenhuma regra cadastrada ainda. Clique em Adicionar resposta TXT para criar a primeira e vincular o arquivo.</p>
  :<>
  <ListTools count={replyRows.length+(replyRows.length===1?' regra':' regras')} total={replyRows.length} shown={replyVisible.length} search={qReplies} onSearch={setQReplies} searchLabel="Filtrar respostas TXT" onExpandAll={()=>setAll(allReplyIds,true)} onCollapseAll={()=>setAll(allReplyIds,false)}/>
  <ItemStack>{replyVisible.map(({r,i})=>{
   const label='Regra '+(i+1);
   return <AccordionItem key={r.id} id={r.id} open={isOpen(r.id)} enabled={r.enabled} enabledLabel={'Ativar '+label} title={r.keyword.trim()||label} meta={[MATCH[r.matching]||'',SELECT[r.selection]||'',assetName(r.asset)||'sem arquivo',r.cooldown+'s']} onToggle={()=>flip(r.id)} onEnabled={enabled=>reply(r.id,{enabled})}>
    <Field label="Palavra ou expressão"><input value={r.keyword} maxLength={100} onChange={e=>reply(r.id,{keyword:e.target.value})}/></Field>
    <div className="form-grid"><Field label="Reconhecer"><select value={r.matching} onChange={e=>reply(r.id,{matching:e.target.value})}><option value="word">Palavra / expressão inteira</option><option value="contains">Qualquer trecho da mensagem</option></select></Field><Field label="Escolher resposta"><select value={r.selection} onChange={e=>reply(r.id,{selection:e.target.value})}><option value="random">Aleatória, sem repetir a última</option><option value="sequence">Em sequência</option></select></Field></div>
    {file('txt',r.asset,asset=>reply(r.id,{asset}))}
    <div className="form-grid"><Field label="Intervalo da regra (segundos)"><input type="number" min="1" max="86400" value={r.cooldown} onChange={e=>reply(r.id,{cooldown:+e.target.value})}/></Field><Field label="Intervalo por pessoa (segundos)"><input type="number" min="0" max="86400" value={r.userCooldown} onChange={e=>reply(r.id,{userCooldown:+e.target.value})}/></Field></div>
    <div className="item-actions"><button disabled={!desktop||!r.asset} onClick={()=>void run(async()=>{setPreview(await api('chatExtras.preview',{profileId,asset:r.asset}))})}>Conferir linhas</button><button className="danger" onClick={()=>setConfig({...config,replies:config.replies.filter(x=>x.id!==r.id)})}>Remover regra</button></div>
   </AccordionItem>})}
  </ItemStack>
  {replyVisible.length===0&&<p className="help">Nenhuma regra corresponde a “{qReplies}”.</p>}
  </>}
 <button onClick={addReply}>Adicionar resposta TXT</button>
 {preview&&<div aria-live="polite"><p>{preview.count} respostas encontradas. Primeiras 10:</p><pre className="variable-output">{preview.lines.join('\n')}</pre></div>}</Card>
 <Card title="Sons por espectador"><Toggle label="Ativar sons por espectador" checked={config.soundsEnabled} onChange={soundsEnabled=>setConfig({...config,soundsEnabled})}/><p className="help">Escolha se cada pessoa deve tocar ao falar, entrar no chat ou em ambos os casos. Na Twitch, a entrada silenciosa exige ativar o monitor abaixo e autorizar novamente a conta do bot. Informe o nome exato do chat ou o ID da plataforma; o apelido serve para sua organização. Áudios são copiados para o BotLive, limitados a 5 MiB e reproduzidos por até 15 segundos. Capture o áudio do BotLive ou do desktop no OBS.</p>
 <Toggle label="Monitorar entradas silenciosas na Twitch" checked={config.presenceEnabled} onChange={presenceEnabled=>setConfig({...config,presenceEnabled})}/><p className="help">Detecta conexões ao chat, não quem está assistindo ao vídeo. A Twitch pode atrasar ou omitir entradas. Pessoas já presentes ao conectar o bot não disparam som. No YouTube, use mensagens; outras plataformas precisam de uma ponte de entrada.</p><AudioOutput value={config.audioDeviceId} onChange={audioDeviceId=>setConfig({...config,audioDeviceId})}/>
 {config.people.length===0
  ?<p className="help">Nenhuma pessoa cadastrada ainda. Clique em Adicionar pessoa para escolher o áudio e definir quando ele toca.</p>
  :<>
  <ListTools count={personRows.length+(personRows.length===1?' pessoa':' pessoas')} total={personRows.length} shown={personVisible.length} search={qPeople} onSearch={setQPeople} searchLabel="Filtrar pessoas" onExpandAll={()=>setAll(allPersonIds,true)} onCollapseAll={()=>setAll(allPersonIds,false)}/>
  <ItemStack>{personVisible.map(({r,i})=>{
   const label='Pessoa '+(i+1);
   return <AccordionItem key={r.id} id={r.id} open={isOpen(r.id)} enabled={r.enabled} enabledLabel={'Ativar som da pessoa '+(i+1)} title={r.nickname.trim()||r.name.trim()||label} meta={[TRIGGER[r.trigger]||'',MODE[r.mode]||'',assetName(r.asset)||'sem som',r.cooldown+'s']} onToggle={()=>flip(r.id)} onEnabled={enabled=>person(r.id,{enabled})}>
    <div className="form-grid"><Field label="Nome no chat"><input value={r.name} maxLength={100} onChange={e=>person(r.id,{name:e.target.value})}/></Field><Field label="Apelido"><input value={r.nickname} maxLength={100} onChange={e=>person(r.id,{nickname:e.target.value})}/></Field></div>
    <Field label="ID da pessoa (opcional)" hint="Quando preenchido, o ID tem prioridade sobre o nome; recomendado para evitar nomes duplicados."><input value={r.userId} maxLength={200} onChange={e=>person(r.id,{userId:e.target.value})}/></Field>
    {file('sound',r.asset,asset=>person(r.id,{asset}))}
    <div className="form-grid"><Field label="Disparar som"><select value={r.trigger} onChange={e=>person(r.id,{trigger:e.target.value})}><option value="message">Quando enviar mensagem</option><option value="join">Quando entrar, mesmo sem falar</option><option value="either">Ao entrar ou enviar mensagem</option></select></Field><Field label="Quando tocar"><select value={r.mode} onChange={e=>person(r.id,{mode:e.target.value})}><option value="first">Uma vez por sessão</option><option value="interval">Repetir com intervalo</option></select></Field><Field label="Intervalo mínimo (segundos)"><input type="number" min="5" max="86400" value={r.cooldown} onChange={e=>person(r.id,{cooldown:+e.target.value})}/></Field></div>
    <Field label={'Volume: '+Math.round(r.volume*100)+'%'}><input type="range" min="0" max="1" step="0.05" value={r.volume} onChange={e=>person(r.id,{volume:+e.target.value})}/></Field>
    <div className="item-actions"><button disabled={!desktop||!r.asset} onClick={()=>void run(async()=>{await playViewerSound({profileId,asset:r.asset,volume:r.volume,deviceId:config.audioDeviceId})})}>Testar som</button><button className="danger" onClick={()=>setConfig({...config,people:config.people.filter(x=>x.id!==r.id)})}>Remover pessoa</button></div>
   </AccordionItem>})}
  </ItemStack>
  {personVisible.length===0&&<p className="help">Nenhuma pessoa corresponde a “{qPeople}”.</p>}
  </>}
  <button onClick={addPerson}>Adicionar pessoa</button><button disabled={!desktop} onClick={()=>void run(async()=>{await api('chatExtras.reset',{profileId});setMessage('Sessão de sons reiniciada. A próxima participação poderá tocar novamente.')})}>Reiniciar sessão de sons</button><p className="help">Há intervalo geral de 5 segundos entre sons. A sessão reinicia ao fechar o app ou pelo botão acima. Mantenha o aplicativo aberto e desbloqueado. Testar som reproduz mesmo com a função desativada.</p></Card>
 <footer className="form-footer"><button className="primary" disabled={!desktop} onClick={()=>void run(async()=>{await api('chatExtras.save',{profileId,config});setMessage('Respostas e sons salvos.')})}>Salvar respostas e sons</button></footer></fieldset>
 {error&&<p role="alert" className="inline-error">{error}</p>}{message&&<p role="status">{message}</p>}</div>
}
