const {test}=require('node:test');
const assert=require('node:assert/strict');
const {publicManifest}=require('./release-manifest.cjs');
const repo='studiocactus/arrobabot';
const url='https://api.github.com/repos/'+repo+'/releases/assets/123';
const publicUrl='https://github.com/'+repo+'/releases/download/v0.1.1/BotLive.exe';
function fixture(){return [{version:'0.1.1',platforms:{'windows-x86_64':{url,signature:'signed'}}},{draft:true,tag_name:'v0.1.1',assets:[{name:'BotLive.exe',url,browser_download_url:publicUrl},{name:'BotLive.exe.sig'}]}];}
test('converte URL da API para asset público e preserva assinatura e original',()=>{
  const [m,r]=fixture(); const out=publicManifest(m,r,'0.1.1',repo);
  assert.equal(out.platforms['windows-x86_64'].url,publicUrl);
  assert.equal(out.platforms['windows-x86_64'].signature,'signed');
  assert.equal(m.platforms['windows-x86_64'].url,url);
  assert.deepEqual(publicManifest(out,r,'0.1.1',repo),out);
});
test('rejeita arquivo desconhecido, destino externo e assinatura ausente',()=>{
  for(const mutate of [m=>m.platforms['windows-x86_64'].url=url+'0',m=>m.platforms['windows-x86_64'].signature='',(m,r)=>r.assets[0].browser_download_url='https://example.com/a.exe',(m,r)=>r.assets.pop()]){
    const [m,r]=fixture();mutate(m,r);assert.throws(()=>publicManifest(m,r,'0.1.1',repo));
  }
});
test('rascunho sem tag criada produz URL da tag definitiva',()=>{
  const [m,r]=fixture();r.assets[0].browser_download_url=publicUrl.replace('v0.1.1','untagged-123');
  assert.equal(publicManifest(m,r,'0.1.1',repo).platforms['windows-x86_64'].url,publicUrl);
});
test('não altera release publicada, tag ou versão diferente e manifesto vazio',()=>{
  for(const mutate of [(m,r)=>r.draft=false,(m,r)=>r.tag_name='v0.1.2',m=>m.version='0.1.2',m=>m.platforms={}]){
    const [m,r]=fixture();mutate(m,r);assert.throws(()=>publicManifest(m,r,'0.1.1',repo));
  }
});
