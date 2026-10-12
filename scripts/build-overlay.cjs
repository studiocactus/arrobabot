// Gera o overlay de meta autocontido a partir do código TypeScript compartilhado
// (src/overlayApp.ts + src/overlayDesign.ts). Assim a prévia do painel, o arquivo
// examples/ e o HTML servido em http://127.0.0.1:PORT/overlay nunca divergem.
const esbuild=require('esbuild');
const fs=require('fs');
const path=require('path');
const root=path.resolve(__dirname,'..');

async function main(){
 const result=await esbuild.build({
  entryPoints:[path.join(root,'src','overlayApp.ts')],
  bundle:true,format:'iife',platform:'browser',target:['chrome100'],
  minify:true,legalComments:'none',write:false,
 });
 const js=result.outputFiles[0].text.replace(/<\/script/g,'<\\/script');
 const template=fs.readFileSync(path.join(root,'scripts','overlay-template.html'),'utf8');
 const html=template.replace('/*__OVERLAY_JS__*/',()=>js);
 if(html===template)throw new Error('Marcador /*__OVERLAY_JS__*/ ausente no template.');
 for(const out of ['src-tauri/overlay.html','examples/goal-overlay.html']){
  fs.writeFileSync(path.join(root,out),html);
  console.log('overlay gerado: '+out);
 }
}

main().catch(e=>{console.error(e);process.exit(1)});
