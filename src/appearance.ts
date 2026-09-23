export function applyAccent(color:string){
 const style=document.documentElement.style;
 if(!/^#[0-9a-f]{6}$/i.test(color)){style.removeProperty('--accent');style.removeProperty('--accent-text');return}
 const rgb=[1,3,5].map(i=>parseInt(color.slice(i,i+2),16)/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4);
 const l=rgb[0]*.2126+rgb[1]*.7152+rgb[2]*.0722;
 style.setProperty('--accent',color);style.setProperty('--accent-text',l>.179?'#000000':'#ffffff');
}
