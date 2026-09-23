// One version and a detailed, immutable update record for every commit.
const fs=require('node:fs');const cp=require('node:child_process');const path=require('node:path');const crypto=require('node:crypto');
const root=path.resolve(__dirname,'..');process.chdir(root);
const git=(...args)=>cp.execFileSync('git',args,{encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
const semver=/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
const greater=(a,b)=>{const x=a.split('.').map(Number),y=b.split('.').map(Number);for(let i=0;i<3;i++)if(x[i]!==y[i])return x[i]>y[i];return false};
const read=(p)=>fs.readFileSync(p,'utf8');const writeJSON=(p,v)=>fs.writeFileSync(p,JSON.stringify(v,null,2)+'\n');
function validateNotes(text){for(const heading of ['## O que mudou','## Como usar','## Validação','## Limitações']){const at=text.indexOf(heading+'\n');if(at<0)throw Error('Notas precisam da seção '+heading);const rest=text.slice(at+heading.length).split(/\n## /)[0].trim();if(rest.length<20||/\b(TODO|PREENCHER)\b/i.test(rest))throw Error('Detalhe a seção '+heading);}if(text.length<300)throw Error('Descreva a atualização com detalhes.');}
function check(readFile,previous,changed){
 const pkg=JSON.parse(readFile('package.json')),lock=JSON.parse(readFile('package-lock.json')),config=JSON.parse(readFile('src-tauri/tauri.conf.json')),meta=JSON.parse(readFile('updates/latest.json'));
 const v=pkg.version;if(!semver.test(v))throw Error('Versão deve ser X.Y.Z');
 const cargo=readFile('src-tauri/Cargo.toml').match(/\[package\][\s\S]*?\nversion = "([^"]+)"/);
 const cargoLock=readFile('src-tauri/Cargo.lock').match(/\[\[package\]\]\nname = "botlive"\nversion = "([^"]+)"/);
 if([lock.version,lock.packages[''].version,config.version,meta.version,cargo?.[1],cargoLock?.[1]].some(x=>x!==v))throw Error('Versões divergentes; use npm run update:prepare.');
 if(meta.notesFile!==`updates/${v}.md`||!/^\d{4}-\d{2}-\d{2}$/.test(meta.date))throw Error('Registro de atualização inválido.');
 const notes=readFile(meta.notesFile);validateNotes(notes);
 if(!readFile('CHANGELOG.md').includes(notes.trim()))throw Error('CHANGELOG desatualizado.');
 if(previous&&!greater(v,previous))throw Error('Todo commit precisa aumentar a versão e detalhar o update.');
 if(changed&&(!changed.includes(meta.notesFile)||!changed.includes('updates/latest.json')))throw Error('O commit não inclui seu registro de atualização.');
 return {v,notes,meta};
}
function previousVersion(ref){try{return JSON.parse(git('show',ref+':package.json')).version}catch{return null}}
function main(){const [command,...args]=process.argv.slice(2);
 if(command==='prepare'){
  const [version,notesPath]=args;if(!semver.test(version||'')||!notesPath)throw Error('Uso: npm run update:prepare -- X.Y.Z caminho/das-notas.md');
  const pkg=JSON.parse(read('package.json'));if(!greater(version,pkg.version))throw Error('Use uma versão maior que '+pkg.version);
  const notes=read(notesPath).replace(/\r/g,'');validateNotes(notes);const dest=`updates/${version}.md`;if(fs.existsSync(dest))throw Error('Esta versão já possui notas; não reescreva releases.');
  fs.mkdirSync('updates',{recursive:true});fs.writeFileSync(dest,`# BotLive ${version}\n\n`+notes.trim()+'\n');
  pkg.version=version;writeJSON('package.json',pkg);const lock=JSON.parse(read('package-lock.json'));lock.version=version;lock.packages[''].version=version;writeJSON('package-lock.json',lock);
  const conf=JSON.parse(read('src-tauri/tauri.conf.json'));conf.version=version;writeJSON('src-tauri/tauri.conf.json',conf);
  fs.writeFileSync('src-tauri/Cargo.toml',read('src-tauri/Cargo.toml').replace(/(\[package\][\s\S]*?\nversion = ")[^"]+/,`$1${version}`));
  fs.writeFileSync('src-tauri/Cargo.lock',read('src-tauri/Cargo.lock').replace(/(\[\[package\]\]\nname = "botlive"\nversion = ")[^"]+/,`$1${version}`));
  writeJSON('updates/latest.json',{version,date:new Date().toISOString().slice(0,10),notesFile:dest});
  const files=fs.readdirSync('updates').filter(f=>semver.test(f.slice(0,-3))&&f.endsWith('.md')).sort((a,b)=>greater(a.slice(0,-3),b.slice(0,-3))?-1:1);
  fs.writeFileSync('CHANGELOG.md','# Histórico de atualizações\n\n'+files.map(f=>read('updates/'+f).trim()).join('\n\n---\n\n')+'\n');
  check(read);console.log('Versão '+version+' preparada. Revise as notas, execute testes e inclua os arquivos no commit.');
 }else if(command==='check'){
  if(args[0]==='--staged'){const files=git('diff','--cached','--name-only').split('\n');check(p=>git('show',':'+p),previousVersion('HEAD'),files);}
  else if(args[0]==='--range'){
   const refs=git('rev-list','--reverse',args[1]||'HEAD').split('\n').filter(Boolean);
   for(const ref of refs){let parent=null;try{parent=git('rev-parse',ref+'^')}catch{};const changed=git('diff-tree','--root','--no-commit-id','--name-only','-r',ref).split('\n');check(p=>git('show',ref+':'+p),parent?previousVersion(parent):null,changed);}
  }else check(read);
  console.log('Versões e registros de atualização válidos.');
 }else if(command==='metadata'){
  const {v,notes}=check(read);if(!process.env.GITHUB_OUTPUT)throw Error('GITHUB_OUTPUT ausente');const delimiter=crypto.randomUUID();fs.appendFileSync(process.env.GITHUB_OUTPUT,`version=${v}\nnotes<<${delimiter}\n${notes}\n${delimiter}\n`);
 }else throw Error('Comando esperado: prepare, check ou metadata');
}
try{main()}catch(e){console.error(e.message);process.exitCode=1}
