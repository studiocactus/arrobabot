// Build an offline manual from the project's authored Markdown subset.
// Supports headings, paragraphs, flat lists, tables, fenced code, links and images.
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const docs = path.join(root, 'docs');
const chapters = [
 ['GUIA-DE-USO.md','Primeiro uso'],['01-INSTALACAO.md','Instalação'],
 ['INTEGRACOES.md','Perfis e conexões'],['03-COMANDOS-E-AUTOMACOES.md','Comandos e automações'],
 ['04-IA-E-MEMORIA.md','IA e memória'],['05-COMUNIDADE.md','Comunidade'],
 ['06-PRESETS-E-APARENCIA.md','Presets e aparência'],['07-OPERACAO-E-BACKUP.md','Acesso e backup'],
 ['API-LOCAL.md','OBS e API local'],['08-SOLUCAO-DE-PROBLEMAS.md','Solução de problemas'],
 ['README.md','Glossário e índice'],['ARQUITETURA.md','Arquitetura'],
 ['DISTRIBUICAO.md','Desenvolvimento'],['VALIDACAO.md','Validação'],['MATRIZ-DE-ACEITE.md','Pendências'],['VARIAVEIS.md','Variáveis'],['ATUALIZACOES.md','Atualizações e commits'],['RESPOSTAS-E-SONS.md','Respostas TXT e sons'],['TIMERS-E-CONTADORES.md','Timers e contadores'],['EXEMPLOS-DE-USO.md','Exemplos práticos'],['09-DISCORD.md','Discord']
].map(([file,title],i)=>({file,title,id:'capitulo-'+(i+1)}));
const groups=[{name:'Comece aqui',ids:[1,20,2,3]},{name:'Configure o bot',ids:[4,19,16,5,18,6,21,9,7]},{name:'Cuide da sua live',ids:[8,17,10,11]},{name:'Referência técnica',ids:[12,13,14,15]}];
const readingOrder=groups.flatMap(g=>g.ids.map(id=>chapters.find(c=>c.id==='capitulo-'+id)));
const escape = s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const tick=String.fromCharCode(96), fence=tick.repeat(3);
const links=[];
function destination(url) {
 if(url==='MANUAL.html')return '#top';
 const match=chapters.find(c=>c.file===url);
 if(match)return '#'+match.id;
 if(!/^(https?:|#)/i.test(url)){
  const resolved=path.resolve(docs,decodeURIComponent(url.split('#')[0]));
  if(!fs.existsSync(resolved))throw Error('Link local ausente: '+url);
 }
 links.push(url);return url;
}
function inline(raw){
 raw=raw.replace(/\\\|/g,'|');
 const tokens=[];
 const hold=s=>'\u0000'+(tokens.push(s)-1)+'\u0000';
 let s=raw.replace(new RegExp(tick+'([^'+tick+']+)'+tick,'g'),(_,v)=>hold('<code>'+escape(v)+'</code>'));
 s=s.replace(/!\[([^\]]*)\]\(([^)]+)\)/g,(_,alt,url)=>hold('<figure><img loading="lazy" src="'+escape("data:image/png;base64,"+fs.readFileSync(path.resolve(docs,url)).toString("base64"))+'" alt="'+escape(alt)+'"><figcaption>'+escape(alt)+'</figcaption></figure>'));
 s=s.replace(/\[([^\]]+)\]\(([^)]+)\)/g,(_,label,url)=>hold('<a href="'+escape(destination(url))+'">'+escape(label)+'</a>'));
 s=escape(s).replace(/\*\*([^*]+)\*\*/g,'<strong>$1</strong>');
 return s.replace(/\u0000(\d+)\u0000/g,(_,i)=>tokens[Number(i)]);
}
function render(md,chapter){
 const lines=md.replace(/\r/g,'').split('\n');const html=[];const toc=[];let i=0;const headingIds=new Map();
 while(i<lines.length){
  let line=lines[i];
  if(!line.trim()){i++;continue;}
  if(line.startsWith(fence)){
   const block=[];i++;while(i<lines.length&&!lines[i].startsWith(fence))block.push(lines[i++]);
   if(i===lines.length)throw Error('Bloco de código sem fechamento: '+chapter.file);
   i++;html.push('<pre><code>'+escape(block.join('\n'))+'</code></pre>');continue;
  }
  const h=/^(#{1,4}) (.+)$/.exec(line);
  if(h){
   const level=Math.min(5,h[1].length+1);
   const slug=h[2].normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/^-|-$/g,'');
   const count=(headingIds.get(slug)||0)+1;headingIds.set(slug,count);
   if(h[1].length===2)toc.push('<li><a href="#'+chapter.id+'--'+slug+(count>1?'-'+count:'')+'">'+escape(h[2])+'</a></li>');
   html.push('<h'+level+' id="'+chapter.id+'--'+slug+(count>1?'-'+count:'')+'">'+inline(h[2])+'</h'+level+'>');i++;continue;
  }
  if(line.startsWith('|')&&/^\|[- :|]+\|$/.test((lines[i+1]||'').trim())){
   const row=s=>s.trim().replace(/^\||\|$/g,'').split(/(?<!\\)\|/).map(v=>v.trim());
   const head=row(line);i+=2;let body='';
   while(i<lines.length&&lines[i].startsWith('|'))body+='<tr>'+row(lines[i++]).map(v=>'<td>'+inline(v)+'</td>').join('')+'</tr>';
   html.push('<div class="table-wrap"><table><thead><tr>'+head.map(v=>'<th scope="col">'+inline(v)+'</th>').join('')+'</tr></thead><tbody>'+body+'</tbody></table></div>');continue;
  }
  const list=/^(-|\d+\.) (.+)$/.exec(line);
  if(list){
   const ordered=list[1]!=='-',tag=ordered?'ol':'ul';let out='<'+tag+(ordered?' start="'+parseInt(list[1],10)+'"':'')+'>';
   while(i<lines.length){const item=/^(-|\d+\.) (.+)$/.exec(lines[i]);if(!item||(item[1]!=='-')!==ordered)break;out+='<li>'+inline(item[2])+'</li>';i++;}
   html.push(out+'</'+tag+'>');continue;
  }
  const paragraph=[line];i++;
  while(i<lines.length&&lines[i].trim()&&!/^(#{1,4} |[-] |\d+\. |\|)/.test(lines[i])&&!lines[i].startsWith(fence))paragraph.push(lines[i++]);
  const content=inline(paragraph.join(' '));html.push(content.startsWith('<figure>')?content:'<p>'+content+'</p>');
 }
 return (toc.length?'<details class="chapter-toc"><summary>Neste capítulo</summary><ul>'+toc.join('')+'</ul></details>':'')+html.join('\n');
}
const content=readingOrder.map(c=>'<article id="'+c.id+'" data-chapter><div class="chapter-label">'+escape(c.title)+'</div>'+render(fs.readFileSync(path.join(docs,c.file),'utf8'),c)+'<a class="back" href="#top">Voltar ao início ↑</a></article>').join('\n');
const nav=groups.map(g=>'<section class="nav-group"><h2>'+escape(g.name)+'</h2>'+g.ids.map(id=>{const c=chapters.find(c=>c.id==='capitulo-'+id);return '<a data-nav="'+c.id+'" href="#'+c.id+'">'+escape(c.title)+'</a>'}).join('')+'</section>').join('');
const css=fs.readFileSync(path.join(docs,'manual.css'),'utf8');
const js=fs.readFileSync(path.join(docs,'manual.js'),'utf8');
const version=JSON.parse(fs.readFileSync(path.join(root,'package.json'),'utf8')).version;
const html='<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>BotLive — Manual de uso</title><style>'+css+'</style></head><body><a class="skip" href="#conteudo">Pular para o conteúdo</a><header id="top"><div class="brand">botlive<span>.</span><small>MANUAL DO PROJETO</small></div><div class="tools"><button id="print" type="button">Imprimir / salvar PDF</button></div></header><div class="layout"><aside><label for="search">Encontrar no manual</label><div class="search-row"><input id="search" type="search" placeholder="Ex.: sorteio, OAuth, backup"><button id="clear" type="button" aria-label="Limpar busca">Limpar</button></div><p id="results" role="status" aria-live="polite"></p><nav aria-label="Capítulos">'+nav+'</nav><p class="aside-note">Leitura offline · versão '+escape(version)+'<br>Os arquivos Markdown são a fonte deste manual.</p></aside><main id="conteudo"><section class="intro"><div class="eyebrow">ESCOLHA UMA TAREFA. CONFIGURE. TESTE.</div><h1>Seu guia para usar<br>o BotLive.</h1><p>Comece pelos exemplos práticos ou vá direto à função que deseja configurar. Cada receita mostra onde clicar, o que preencher e qual resultado esperar. A referência técnica fica no final.</p><div class="intro-links"><a href="#capitulo-1">Começar pelo primeiro uso →</a><a href="#capitulo-20">Ver exemplos práticos →</a><a href="#capitulo-19">Criar timers e contadores →</a><a href="#capitulo-10">Resolver um problema</a></div><div class="note">O aplicativo funciona no desktop. A prévia de navegador não executa conexões ou automações. Recursos dependentes de contas reais e pendências de implementação estão identificados nos capítulos.</div></section><p id="empty" hidden>Nenhum capítulo encontrado. Tente outra palavra ou limpe a busca.</p>'+content+'</main></div><footer>BotLive '+escape(version)+' · Documentação em português · Não requer conexão para leitura; referências externas precisam de internet.</footer><script>'+js+'</script></body></html>';
fs.writeFileSync(path.join(docs,'MANUAL.html'),html);
fs.mkdirSync(path.join(root,'entrega'),{recursive:true});
fs.writeFileSync(path.join(root,'entrega','MANUAL-BOTLIVE.html'),html);
console.log('Manual gerado: '+chapters.length+' capítulos, '+(Buffer.byteLength(html)/1024).toFixed(0)+' KiB.');
