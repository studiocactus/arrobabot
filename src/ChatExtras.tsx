import {useCallback,useEffect,useState} from 'react';
import {open} from '@tauri-apps/plugin-dialog';
import {api,desktop,errorText} from './api';
import {AccordionItem,Card,Field,ItemStack,ListTools,Toggle} from './components';
import {secondsToMinutes} from './types';
type Asset={id:string;kind:string;name:string;issues?:Issue[]};
type Issue={line:number;marker:string;suggestion:string|null};
type Reply={id:string;enabled:boolean;keyword:string;matching:string;selection:string;asset:string;cooldown:number;userCooldown:number;reply:boolean;sound:string;soundVolume:number};
/** A config salva junta respostas e sons por espectador; esta tela mexe só nas respostas. */
type Config={repliesEnabled:boolean;replies:Reply[];soundsEnabled:boolean;presenceEnabled:boolean;people:unknown[]};
const MATCH:Record<string,string>={word:'Palavra inteira',contains:'Contém trecho'};
const SELECT:Record<string,string>={random:'Aleatória',sequence:'Sequência'};
const blank:Config={repliesEnabled:false,replies:[],soundsEnabled:false,presenceEnabled:false,people:[]};
export default function ChatExtras({profileId}:{profileId:string}){
 const [config,setConfig]=useState<Config>(blank);const [sounds,setSounds]=useState<Asset[]>([]);const [assets,setAssets]=useState<Asset[]>([]);const [error,setError]=useState('');const [message,setMessage]=useState('');const [busy,setBusy]=useState(false);const [loaded,setLoaded]=useState(!desktop);
 const [preview,setPreview]=useState<{count:number;lines:string[];issues?:Issue[]}|null>(null);
 const [openIds,setOpenIds]=useState<Record<string,boolean>>({});
 const [qReplies,setQReplies]=useState('');
 useEffect(()=>{let live=true;if(desktop)api<{config:Config;assets:Asset[]}>('chatExtras.get',{profileId}).then(v=>{if(live){setConfig({...blank,...v.config,replies:(v.config?.replies||[]).map(r=>({...r,reply:r.reply!==false,sound:r.sound||'',soundVolume:typeof r.soundVolume==='number'?r.soundVolume:1}))});setAssets(v.assets||[]);setSounds((v.assets||[]).filter(a=>a.kind==='sound'));setLoaded(true)}}).catch(e=>{if(live)setError(errorText(e))});return()=>{live=false}},[profileId]);
 async function run(fn:()=>Promise<void>){setBusy(true);setError('');setMessage('');try{await fn()}catch(e){setError(errorText(e))}finally{setBusy(false)}}
 const chooseTxt=useCallback(async(update:(asset:string)=>void)=>{await run(async()=>{
  const path=await open({multiple:false,filters:[{name:'Respostas UTF-8',extensions:['txt']}]});if(typeof path!=='string')return;
  const asset=await api<Asset>('chatExtras.import',{profileId,kind:'txt',path});setAssets(a=>a.concat(asset));update(asset.id);
  const p=await api<{count:number;lines:string[];issues?:Issue[]}>('chatExtras.preview',{profileId,asset:asset.id});setPreview(p);setMessage(p.issues?.length?`TXT vinculado com ${p.issues.length} variáveis para conferir abaixo.`:'TXT vinculado e conferido: nenhuma variável estranha.');
 })},[profileId]);
 const reply=(id:string,patch:Partial<Reply>)=>setConfig(c=>c&&({...c,replies:c.replies.map(r=>r.id===id?{...r,...patch}:r)}));
 const assetName=(id:string)=>assets.find(a=>a.id===id)?.name||'';
 const isOpen=(id:string)=>!!openIds[id];
 const flip=(id:string)=>setOpenIds(o=>({...o,[id]:!o[id]}));
 const setAll=(ids:string[],value:boolean)=>setOpenIds(o=>{const next={...o};ids.forEach(id=>next[id]=value);return next});
 const withItem=(id:string)=>setOpenIds(o=>({...o,[id]:true}));
 const addReply=()=>{const id=crypto.randomUUID();withItem(id);setConfig(c=>({...c,replies:c.replies.concat({id,enabled:true,keyword:'',matching:'word',selection:'random',asset:'',cooldown:30,userCooldown:60,reply:true,sound:'',soundVolume:1})}))};
 const replyRows=config.replies.map((r,i)=>({r,i}));
 const replyVisible=replyRows.filter(({r,i})=>!qReplies||`${r.keyword} ${assetName(r.asset)} regra ${i+1}`.toLowerCase().includes(qReplies.toLowerCase()));
 const allReplyIds=replyVisible.map(({r})=>r.id);
 return <div className="form-pad"><p>Regras de resposta em texto: quando o chat falar uma palavra, o bot responde com uma frase do seu arquivo TXT. Para tocar um som junto, escolha um áudio da biblioteca direto na regra. As alterações entram em vigor ao clicar em Salvar respostas.</p>{!desktop&&<p className="notice">Prévia da configuração. Arquivos, reprodução e salvamento estão disponíveis no aplicativo desktop.</p>}
 <fieldset disabled={busy||!loaded} style={{border:0,padding:0,minWidth:0}}>
 <Card title="Respostas por arquivo TXT"><Toggle label="Ativar respostas TXT" checked={config.repliesEnabled} onChange={repliesEnabled=>setConfig({...config,repliesEnabled})}/><p className="help">Uma resposta por linha, até 450 caracteres. Linhas vazias são ignoradas. Use UTF-8. Alterações no arquivo original valem no próximo disparo. A primeira regra disponível responde; outras automações não executam nessa mensagem. Clique no título da regra para abrir ou fechar o formulário.</p>
 {config.replies.length===0
  ?<p className="help">Nenhuma regra cadastrada ainda. Clique em Adicionar resposta TXT para criar a primeira e vincular o arquivo.</p>
  :<>
  <ListTools count={replyRows.length+(replyRows.length===1?' regra':' regras')} total={replyRows.length} shown={replyVisible.length} search={qReplies} onSearch={setQReplies} searchLabel="Filtrar respostas TXT" onExpandAll={()=>setAll(allReplyIds,true)} onCollapseAll={()=>setAll(allReplyIds,false)}/>
  <ItemStack>{replyVisible.map(({r,i})=>{
   const label='Regra '+(i+1);
   return <AccordionItem key={r.id} id={r.id} open={isOpen(r.id)} enabled={r.enabled} enabledLabel={'Ativar '+label} title={r.keyword.trim()||label} meta={[MATCH[r.matching]||'',SELECT[r.selection]||'',assetName(r.asset)||'sem arquivo',r.sound?assetName(r.sound)||'som':'',secondsToMinutes(r.cooldown)+' min']} onToggle={()=>flip(r.id)} onEnabled={enabled=>reply(r.id,{enabled})}>
    <Field label="Palavra ou expressão" hint="O que o espectador precisa falar para esta resposta valer."><input value={r.keyword} maxLength={100} onChange={e=>reply(r.id,{keyword:e.target.value})}/></Field>
    <div className="form-grid"><Field label="Reconhecer" hint="Palavra inteira exige a mensagem bater no termo; trecho aceita a palavra dentro de qualquer frase."><select value={r.matching} onChange={e=>reply(r.id,{matching:e.target.value})}><option value="word">Palavra / expressão inteira</option><option value="contains">Qualquer trecho da mensagem</option></select></Field><Field label="Escolher resposta" hint="Sequência perde o TXT linha a linha; aleatória pula a repetição da última."><select value={r.selection} onChange={e=>reply(r.id,{selection:e.target.value})}><option value="random">Aleatória, sem repetir a última</option><option value="sequence">Em sequência</option></select></Field></div>
    <Field label="Arquivo de respostas" hint="TXT com uma frase por linha que o bot envia ao chat."><div className="row"><select aria-label="Arquivo de respostas" value={r.asset} onChange={e=>reply(r.id,{asset:e.target.value})}><option value="">Escolha um arquivo</option>{assets.filter(a=>a.kind==='txt').map(a=><option key={a.id} value={a.id}>{a.name}</option>)}</select><button disabled={!desktop||busy} onClick={()=>void chooseTxt(asset=>reply(r.id,{asset}))}>Vincular TXT</button></div></Field>
    <div className="form-grid"><Field label="Intervalo da regra (segundos)" hint="60 = 1 minuto; no mínimo 60."><input type="number" min="60" max="86400" step="5" value={r.cooldown} onChange={e=>reply(r.id,{cooldown:Math.max(60,Number(e.target.value)||60)})}/></Field><Field label="Intervalo por pessoa (segundos)" hint="60 = 1 minuto. 0 sem limite."><input type="number" min="0" max="86400" step="5" value={r.userCooldown} onChange={e=>reply(r.id,{userCooldown:Number(e.target.value)||0})}/></Field></div>
    <div className="list-row"><div><strong>Responder no fio da pessoa</strong><small>Na Twitch a resposta sai no fio de quem falou; no Discord, citando a mensagem.</small></div><Toggle label="Responder no fio da pessoa" checked={r.reply!==false} onChange={v=>reply(r.id,{reply:v})}/></div>
    <Field label="Som junto com a resposta (opcional)" hint="Um áudio da biblioteca toca na saída dos sons do bot no mesmo instante da resposta. Escolha um som importado na tela Sons de Live.">
     <div className="row"><select aria-label="Som junto com a resposta" value={r.sound} onChange={e=>reply(r.id,{sound:e.target.value})}><option value="">Sem som</option>{sounds.map(s=><option key={s.id} value={s.id}>{s.name}</option>)}</select><button disabled={!desktop||!r.sound} onClick={()=>void run(async()=>{await api('audio.preview',{profileId,asset:r.sound,volume:r.soundVolume})})}>Testar som</button></div>
    </Field>
    {r.sound&&<Field label={'Volume do som: '+Math.round(r.soundVolume*100)+'%'}><input type="range" min="0" max="1" step="0.05" value={r.soundVolume} onChange={e=>reply(r.id,{soundVolume:+e.target.value})}/></Field>}
    <div className="item-actions"><button disabled={!desktop||!r.asset} onClick={()=>void run(async()=>{setPreview(await api('chatExtras.preview',{profileId,asset:r.asset}))})}>Conferir linhas</button><button className="danger" onClick={()=>setConfig({...config,replies:config.replies.filter(x=>x.id!==r.id)})}>Remover regra</button></div>
   </AccordionItem>})}
  </ItemStack>
  {replyVisible.length===0&&<p className="help">Nenhuma regra corresponde a “{qReplies}”.</p>}
  </>}
 <button onClick={addReply}>Adicionar resposta TXT</button>
 {preview&&<div aria-live="polite"><p>{preview.count} respostas encontradas. Primeiras 10:</p><pre className="variable-output">{preview.lines.join('\n')}</pre>{preview.issues&&preview.issues.length>0&&<div className="inline-warning"><strong>Variáveis para conferir:</strong>{preview.issues.map((w,i)=><p key={i}>Linha {w.line}: <code>{`{{${w.marker}}}`}</code> não existe{w.suggestion?<> — quis dizer <code>{`{{${w.suggestion}}}`}</code>?</>:'.'}</p>)}</div>}</div>}</Card>
 <footer className="form-footer"><button className="primary" disabled={!desktop} onClick={()=>void run(async()=>{await api('chatExtras.save',{profileId,config});setMessage('Respostas salvas.')})}>Salvar respostas</button></footer></fieldset>
 {error&&<p role="alert" className="inline-error">{error}</p>}{message&&<p role="status">{message}</p>}</div>
}
