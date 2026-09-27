import {useLayoutEffect} from 'react';
/** Deixa o quadro crescer com o texto: uma instrução longa de IA não fica rolando numa caixa fixa.
 * A altura volta a ser a medida do conteúdo a cada tecla; a classe `auto-grow` limita o teto. */
export function useAutoGrow(ref:{current:HTMLTextAreaElement|null},value:string){
 useLayoutEffect(()=>{
  const el=ref.current;if(!el)return;
  el.style.height='auto';
  el.style.height=(el.scrollHeight+el.offsetHeight-el.clientHeight)+'px';
 },[ref,value]);
}
