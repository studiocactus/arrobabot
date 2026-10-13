/**
 * Modo código — HTML, CSS e JS do próprio streamer sobre a camada do overlay.
 * A mesma injeção roda na fonte do OBS e na prévia do Estúdio, então o que se
 * vê no Estúdio é o que o OBS desenha (WYSIWYG).
 *
 * O CSS é prefixado com o escopo do contêiner (body no overlay, a caixa de
 * prévia no Estúdio): no aplicativo ele não pinta a interface, e no OBS vale a
 * cena inteira. O JS roda com `root` = a própria camada (o app permite eval
 * via CSP; o OBS não restringe).
 */

export type CodeBlock={html:string;css:string;js:string};

function matchBrace(src:string,open:number):number{
 let depth=0;
 for(let i=open;i<src.length;i++){
  if(src[i]==='{')depth++;
  else if(src[i]==='}'){
   depth--;
   if(depth===0)return i;
  }
 }
 return src.length;
}

/**
 * Prefixa cada seletor do CSS com o escopo: `.ch-stage .bl-chat-name{…}`.
 * Regras dentro de @media/@supports/@layer/@container são prefixadas também;
 * @keyframes, @font-face e afins ficam intactos. `:root`, `html` e `body`
 * viram o próprio escopo — é assim que se pinta a "janela" do widget.
 */
export function scopeCss(css:string,scope:string):string{
 const clean=css.replace(/\/\*[\s\S]*?\*\//g,'');
 let out='';
 let i=0;
 while(i<clean.length){
  const open=clean.indexOf('{',i);
  if(open<0){out+=clean.slice(i);break}
  const sel=clean.slice(i,open).trim();
  const close=matchBrace(clean,open);
  const body=clean.slice(open+1,close);
  if(sel.startsWith('@')){
   out+=/^(?:@media|@supports|@layer|@container)\b/.test(sel)
    ?`${sel}{${scopeCss(body,scope)}}`
    :`${sel}{${body}}`;
  }else{
   const rules=sel.split(',').map(part=>{
    // `:root`/`html`/`body` no início viram o próprio escopo; o resto é que
    // precisa dele na frente.
    let t=part.trim();
    while(/^(?:html|body|:root)(?![\w-])\s*/.test(t))t=t.replace(/^(?:html|body|:root)(?![\w-])\s*/,'');
    return t?`${scope} ${t}`:scope;
   }).filter(Boolean).join(',');
   if(rules)out+=`${rules}{${body}}`;
  }
  i=close+1;
 }
 return out;
}

/** Escopo do contêiner: body no overlay, o elemento da prévia no Estúdio. */
export function cssScope(el:HTMLElement):string{
 if(el===document.body)return 'body';
 if(el.id)return '#'+el.id;
 return el.classList.length?'.'+el.classList[0]:'body';
}

type Applied={html:string;css:string;js:string};
const applied=new WeakMap<HTMLElement,Applied>();
const CHANGED='\u0000';

/**
 * Aplica (ou reaplica) o modo código na camada. `ns` separa chat de alertas.
 * Cada bloco só é refeito quando muda — o JS não roda de novo a cada tecla.
 */
export function applyCode(scope:HTMLElement,code:CodeBlock,ns:string):void{
 const prev=applied.get(scope)||{html:CHANGED,css:CHANGED,js:CHANGED};
 if(prev.css!==code.css){
  document.getElementById(`bl-code-${ns}`)?.remove();
  if(code.css){
   const st=document.createElement('style');
   st.id=`bl-code-${ns}`;
   st.textContent=scopeCss(code.css,cssScope(scope));
   document.head.appendChild(st);
  }
 }
 if(prev.html!==code.html||!scope.querySelector(`[data-bl-code="${ns}"]`)){
  scope.querySelectorAll(`[data-bl-code="${ns}"]`).forEach(n=>n.remove());
  if(code.html){
   const holder=document.createElement('div');
   holder.setAttribute('data-bl-code',ns);
   holder.innerHTML=code.html;
   // script via innerHTML não executa; fora evita deixar nó morto no DOM.
   holder.querySelectorAll('script').forEach(s=>s.remove());
   scope.prepend(holder);
  }
 }
 if(prev.js!==code.js&&code.js){
  try{
   // eslint-disable-next-line no-new-func
   new Function('root',code.js)(scope);
  }catch(err){
   console.warn('[BotLive] JS do modo código falhou:',err);
  }
 }
 applied.set(scope,{html:code.html,css:code.css,js:code.js});
}
