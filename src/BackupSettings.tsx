import {useEffect,useState} from 'react';
import {open} from '@tauri-apps/plugin-dialog';
import {FolderOpen,Save,FileUp,Archive} from 'lucide-react';
import {api,desktop,errorText} from './api';
import {Card,Field,Toggle} from './components';

type Config={folder:string;auto:boolean;days:number;weeks:number;months:number;lastSuccess:number};
type Entry={name:string;path:string;date:string;size:number};
const empty:Config={folder:'',auto:false,days:7,weeks:4,months:12,lastSuccess:0};
const mb=(bytes:number)=>(bytes/1024/1024).toFixed(1).replace('.',',')+' MB';

/** Backup de todas as áreas em um arquivo .botlivebak, com retenção por dia, semana e mês. */
export default function BackupSettings({notify}:{notify:(s:string)=>void}){
 const [config,setConfig]=useState<Config>(empty);
 const [files,setFiles]=useState<Entry[]>([]);
 const [busy,setBusy]=useState(false);
 async function refresh(){
  try{
   const c=await api<Partial<Config>|null>('backup.get');
   setConfig({...empty,...(c&&typeof c==='object'?c:empty)});
   const listed=await api<Entry[]|null>('backup.list');
   setFiles(Array.isArray(listed)?listed:[]);
  }catch(e){notify(errorText(e))}
 }
 useEffect(()=>{if(desktop)void refresh()},[]);
 async function save(patch:Partial<Config>){
  if(!desktop){notify('Abra o aplicativo desktop para configurar o backup.');return}
  const current=config;const next={...current,...patch};setConfig(next);setBusy(true);
  try{const saved=await api<Partial<Config>|null>('backup.save',{config:next});setConfig({...empty,...(saved&&typeof saved==='object'?saved:next)});notify('Ajustes de backup salvos.')}catch(e){setConfig(current);notify(errorText(e))}finally{setBusy(false)}
 }
 async function chooseFolder(){
  try{
   const dir=await open({directory:true,multiple:false,title:'Escolha a pasta dos backups'});
   if(typeof dir==='string')await save({folder:dir});
  }catch(e){notify(errorText(e))}
 }
 async function backupNow(){
  setBusy(true);
  try{
   const r=await api<{name?:string;size?:number;pruned?:number}|null>('backup.now',{folder:config.folder});
   const nome=r&&r.name?r.name:'backup.botlivebak';
   notify(`Backup salvo: ${nome}${r&&r.size?` (${mb(r.size)})`:''}.`+(r&&r.pruned?` ${r.pruned} arquivo(s) antigo(s) fora da retenção.`:''));
   await refresh();
  }catch(e){notify(errorText(e))}finally{setBusy(false)}
 }
 async function restore(){
  if(!desktop){notify('Abra o aplicativo desktop para importar um backup.');return}
  const file=await open({multiple:false,title:'Escolha o arquivo de backup',filters:[{name:'Backup do BotLive',extensions:['botlivebak']}]});
  if(typeof file!=='string')return;
  const sure=window.confirm('Importar este backup substitui TODAS as áreas do bot por as do arquivo: perfis, comandos, timers, automações, contadores, histórico, presets, ajustes, pontos, variáveis, memórias, base de conhecimento e sons. O BotLive grava uma cópia de segurança antes. As credenciais não mudam. Continuar?');
  if(!sure)return;
  setBusy(true);
  try{
   const r=await api<{profiles?:number;flows?:number;files?:number}|null>('backup.restore',{path:file});
   notify(r&&r.profiles!=null?`Backup importado: ${r.profiles} perfil(s), ${r.flows||0} automação(ões), ${r.files||0} arquivo(s).`:'Backup importado.');
   setTimeout(()=>location.reload(),700);
  }catch(e){notify(errorText(e))}finally{setBusy(false)}
 }
 return <Card title="Backup do bot" action={<Archive size={16}/>}>
 <p className="help">Cada backup é um arquivo <code>.botlivebak</code> com todas as áreas do bot: perfis, comandos, timers, automações, contadores, histórico, presets, ajustes, pontos, variáveis, memórias, base de conhecimento e sons. O cofre de credenciais fica no computador e nunca entra no arquivo.</p>
 <Field label="Pasta dos backups" hint="Escolha uma pasta sua, de preferência fora da pasta de dados.">
  <div className="row"><input readOnly value={config.folder||'Nenhuma pasta escolhida'} aria-label="Pasta dos backups"/><button onClick={()=>void chooseFolder()} disabled={busy}><FolderOpen size={15}/>Escolher pasta</button></div>
 </Field>
 <div className="switch-row"><span>Backup automático diário</span><Toggle label="Backup automático diário" checked={!!config.auto} onChange={auto=>void save({auto})}/></div>
 <p className="help">Com o automático ligado, um arquivo por dia é gravado quando o aplicativo está aberto. Também dá para clicar em <strong>Fazer backup agora</strong> antes de fechar antes de uma live.</p>
 <div className="form-grid">
  <Field label="Guardar por dias"><input type="number" min="1" max="365" value={config.days} onChange={e=>setConfig({...config,days:+e.target.value})} onBlur={()=>void save({days:config.days})}/></Field>
  <Field label="Semanas"><input type="number" min="1" max="52" value={config.weeks} onChange={e=>setConfig({...config,weeks:+e.target.value})} onBlur={()=>void save({weeks:config.weeks})}/></Field>
  <Field label="Meses"><input type="number" min="1" max="60" value={config.months} onChange={e=>setConfig({...config,months:+e.target.value})} onBlur={()=>void save({months:config.months})}/></Field>
 </div>
 <p className="help">Valem as últimas cópias de cada dia do período, a mais recente de cada semana e a mais recente de cada mês. O resto é apagado sozinho, e só arquivos com o nome do BotLive são tocados.</p>
 <div className="row">
  <button className="primary" disabled={busy||!config.folder} onClick={()=>void backupNow()}><Save size={15}/>Fazer backup agora</button>
  <button disabled={busy} onClick={()=>void restore()}><FileUp size={15}/>Importar backup…</button>
 </div>
 {files.length>0&&<details><summary>{files.length} arquivo(s) na pasta</summary>
  {files.map(f=><div className="list-row" key={f.path}><div><strong>{f.name}</strong><small>{f.date}</small></div><span>{mb(f.size)}</span></div>)}
 </details>}
 {config.lastSuccess>0&&<p className="help">Último backup automático: {new Date(config.lastSuccess*1000).toLocaleString('pt-BR')}.</p>}
 {!desktop&&<p className="help">Esta prévia não grava arquivos: use o aplicativo desktop.</p>}
 </Card>
}
