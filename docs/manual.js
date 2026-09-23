const input=document.getElementById('search');
const articles=Array.from(document.querySelectorAll('[data-chapter]'));
const norm=s=>s.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLowerCase();
const index=articles.map(el=>({el,text:norm(el.textContent)}));
function filter(){
 const words=norm(input.value).trim().split(/\s+/).filter(Boolean);let found=0;
 for(const item of index){const match=words.every(w=>item.text.includes(w));item.el.hidden=!match;document.querySelector('[data-nav="'+item.el.id+'"]').hidden=!match;if(match)found++}
 document.getElementById('results').textContent=words.length?found+' de '+articles.length+' capítulos encontrados':articles.length+' capítulos · leia na ordem que precisar';
 document.getElementById('empty').hidden=found!==0;
}
function clearSearch(){input.value='';filter()}
input.addEventListener('input',filter);
document.getElementById('clear').addEventListener('click',()=>{clearSearch();input.focus()});
document.getElementById('print').addEventListener('click',()=>{clearSearch();window.print()});
document.addEventListener('click',e=>{const link=e.target.closest('a[href^="#"]');if(!link)return;const target=document.getElementById(link.hash.slice(1));if(target&&target.closest('[hidden]'))clearSearch()});
window.addEventListener('hashchange',()=>{const target=document.getElementById(location.hash.slice(1));if(target&&target.hidden)clearSearch()});
filter();
