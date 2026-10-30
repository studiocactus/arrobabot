import {useEffect,useState} from 'react';
import {api,desktop} from './api';
import type {Profile} from './types';

/** A identidade é sempre a do canal do perfil ativo, nunca a conta do bot. */
export default function AccountAvatar({profile,online}:{profile?:Profile;online:boolean}){
 const [account,setAccount]=useState<{imageUrl:string;displayName:string}|null>(null);
 const [failed,setFailed]=useState(false);
 const id=profile?.id,channel=profile?.channelId,platform=profile?.platform,client=profile?.clientId;
 useEffect(()=>{
  let active=true;setAccount(null);setFailed(false);
  if(!desktop||!id||!channel||platform!=='twitch')return;
  async function refresh(){try{const next=await api<{imageUrl:string;displayName:string}>('twitch.identity',{profileId:id});if(active){setAccount(next);setFailed(false)}}catch{if(active)setAccount(null)}}
  void refresh();const timer=setInterval(()=>void refresh(),300000);
  return()=>{active=false;clearInterval(timer)};
 },[id,channel,platform,client,online]);
 const label=account?.displayName||profile?.channel||profile?.name||'BotLive';
 return <span className="top-avatar" title={label}>{account?.imageUrl&&!failed?<img src={account.imageUrl} alt={`Foto de ${label} na Twitch`} referrerPolicy="no-referrer" onError={()=>setFailed(true)}/>:<span aria-label={`Conta: ${label}`}>{profile?.name.slice(0,1).toUpperCase()||'B'}</span>}</span>;
}
