// Gera os HTML autocontidos a partir do código TypeScript compartilhado:
// o overlay visual (src/overlayApp.ts + src/overlayDesign.ts) e a fonte de
// áudio (src/audioSourceApp.ts). Assim a prévia do painel, o arquivo
// examples/ e o que o servidor local entrega nunca divergem.
const esbuild=require('esbuild');
const fs=require('fs');
const path=require('path');
const root=path.resolve(__dirname,'..');

async function build(entry,marker,template,outs){
 const result=await esbuild.build({
  entryPoints:[path.join(root,'src',entry)],
  bundle:true,format:'iife',platform:'browser',target:['chrome100'],
  minify:true,legalComments:'none',write:false,
 });
 const js=result.outputFiles[0].text.replace(/<\/script/g,'<\\/script');
 const tpl=fs.readFileSync(path.join(root,'scripts',template),'utf8');
 const html=tpl.replace(marker,()=>js);
 if(html===tpl)throw new Error(`Marcador ${marker} ausente no template ${template}.`);
 for(const out of outs){
  fs.writeFileSync(path.join(root,out),html);
  console.log('gerado: '+out);
 }
}

async function main(){
 await build('overlayApp.ts','/*__OVERLAY_JS__*/','overlay-template.html',['src-tauri/overlay.html','examples/goal-overlay.html']);
 await build('audioSourceApp.ts','/*__AUDIO_JS__*/','audio-template.html',['src-tauri/audio.html']);
}

main().catch(err=>{console.error(err);process.exit(1)});
