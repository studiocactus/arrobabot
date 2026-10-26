import {useCallback,useEffect} from 'react';
export const navigationAllowed=()=>window.dispatchEvent(new Event('botlive:navigate',{cancelable:true}));
export default function useUnsavedChanges(dirty:boolean){
 const confirmLeave=useCallback(()=>!dirty||window.confirm('Há alterações não salvas. Sair e descartar essas alterações?'),[dirty]);
 useEffect(()=>{
  if(!dirty)return;
  const unload=(event:BeforeUnloadEvent)=>{event.preventDefault();event.returnValue='';};
  const navigate=(event:Event)=>{if(!confirmLeave())event.preventDefault();};
  window.addEventListener('beforeunload',unload);
  window.addEventListener('botlive:navigate',navigate);
  return()=>{window.removeEventListener('beforeunload',unload);window.removeEventListener('botlive:navigate',navigate);};
 },[dirty,confirmLeave]);
 return confirmLeave;
}
