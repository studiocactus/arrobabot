import {expect,it,vi} from 'vitest';
import {SourcePlayer,MAX_AGE_MS} from './audioSource';

type Sent={type?:string;id?:string;phase?:string;detail?:string};
class FakeSound {
 volume=0;onended:(()=>void)|null=null;onerror:(()=>void)|null=null;
 pause=vi.fn();play=vi.fn(async()=>{});
 constructor(public src:string){}
}

function setup(now=1_000_000){
 const sent:Sent[]=[];
 const sounds:FakeSound[]=[];
 const onBlocked=vi.fn();
 const player=new SourcePlayer({
  profile:'p1',
  send:m=>sent.push(m as Sent),
  now:()=>now,
  create:src=>{const s=new FakeSound(src);sounds.push(s);return s},
  onBlocked,
 });
 return {sent,sounds,player,onBlocked};
}

const play=(id:string,extra:Record<string,unknown>={})=>({
 type:'audio.play',
 payload:{profileId:'p1',id,src:`http://127.0.0.1:9876/asset?token=t&id=${id}`,volume:0.5,issuedAt:1_000_000,...extra},
});
const flush=()=>new Promise(r=>setTimeout(r,0));

it('toca o evento e confirma que a reprodução começou de verdade',async()=>{
 const {sent,sounds,player}=setup();
 player.handle(play('a1'));
 await flush();
 expect(sounds).toHaveLength(1);
 expect(sounds[0].volume).toBe(0.5);
 await flush();
 expect(sent).toContainEqual({type:'audio.ack',id:'a1',phase:'started'});
 sounds[0].onended?.();
 expect(sent).toContainEqual({type:'audio.ack',id:'a1',phase:'done'});
});

it('serializa: o próximo som espera o atual terminar',async()=>{
 const {sounds,player}=setup();
 player.handle(play('a1'));
 await flush();
 expect(sounds,'um som por vez').toHaveLength(1);
 player.handle(play('a2'));
 await flush();
 expect(sounds,'o segundo espera na fila').toHaveLength(1);
 sounds[0].onended?.();await flush();
 expect(sounds).toHaveLength(2);
});

it('descarta evento antigo e confirma como erro — reconexão não repete fila',async()=>{
 const {sent,sounds,player}=setup();
 player.handle({type:'audio.play',payload:{profileId:'p1',id:'velho',src:'http://127.0.0.1:9/a.wav',issuedAt:1_000_000-(MAX_AGE_MS+1000)}});
 expect(sounds).toHaveLength(0);
 expect(sent[0]).toMatchObject({id:'velho',phase:'error',detail:'Som antigo descartado'});
});

it('ignora evento de outro perfil e endereço que não seja local',async()=>{
 const {sounds,player}=setup();
 player.handle({type:'audio.play',payload:{profileId:'outro',id:'x',src:'http://127.0.0.1:9/a.wav',issuedAt:1_000_000}});
 player.handle({type:'audio.play',payload:{profileId:'p1',id:'y',src:'https://evil.example.com/a.mp3',issuedAt:1_000_000}});
 player.handle({type:'audio.play',payload:{profileId:'p1',id:'z',src:'data:text/html,<script>alert(1)</script>',issuedAt:1_000_000}});
 expect(sounds,'nada toca: nem outro perfil, nem rede, nem HTML').toHaveLength(0);
});

it('fila cheia avisa em vez de acumular',async()=>{
 const {sent,sounds,player}=setup();
 for(let i=0;i<5;i++){player.handle(play(`f${i}`));await flush()}
 expect(sounds).toHaveLength(1);
 player.handle(play('extra'));
 expect(sent.some(m=>m.phase==='error'&&m.detail==='Fila de sons cheia')).toBe(true);
});

it('volume acima de 1 é cortado e volume ausente vira 100%',async()=>{
 const {sounds,player}=setup();
 player.handle(play('v1',{volume:3}));
 player.handle(play('v2',{volume:undefined}));
 await flush();sounds[0].onended?.();await flush();
 expect(sounds[0].volume).toBe(1);
 expect(sounds[1].volume).toBe(1);
});

it('parar tudo pausa o som atual, emite stopped e esvazia a fila',async()=>{
 const {sent,sounds,player}=setup();
 player.handle(play('a1'));
 player.handle(play('a2'));
 await flush();
 player.handle({type:'audio.stop'});
 expect(sounds[0].pause).toHaveBeenCalled();
 expect(sent).toContainEqual({type:'audio.ack',id:'a1',phase:'stopped'});
 await flush();await flush();
 expect(sounds,'o som da fila nunca toca depois de parar').toHaveLength(1);
});

it('falha do navegador vira erro confirmado e avisa o bloqueio',async()=>{
 const sent:Sent[]=[];
 const onBlocked=vi.fn();
 const blocked=Object.assign(new Error('nope'),{name:'NotAllowedError'});
 const player=new SourcePlayer({
  profile:'p1',
  send:m=>sent.push(m as Sent),
  now:()=>1_000_000,
  create:()=>({src:'',volume:0,pause(){},onended:null,onerror:null,play:vi.fn(async()=>{throw blocked})} as never),
  onBlocked,
 });
 player.handle({type:'audio.play',payload:{profileId:'p1',id:'b1',src:'http://127.0.0.1:9/a.wav',issuedAt:1_000_000}});
 await flush();
 expect(sent.some(m=>m.id==='b1'&&m.phase==='error'&&m.detail==='Reprodução bloqueada pelo navegador')).toBe(true);
 expect(onBlocked).toHaveBeenCalled();
});

it('carregamento quebrado confirma erro e libera a fila',async()=>{
 const {sent,sounds,player}=setup();
 player.handle(play('e1'));
 player.handle(play('e2'));
 await flush();
 sounds[0].onerror?.();
 expect(sent).toContainEqual({type:'audio.ack',id:'e1',phase:'error',detail:'Falha ao carregar o áudio'});
 await flush();
 expect(sounds).toHaveLength(2);
});

it('teto de 15 s interrompe o som e confirma erro',async()=>{
 vi.useFakeTimers();
 const {sent,sounds,player}=setup();
 player.handle(play('t1'));
 await vi.advanceTimersByTimeAsync(15_000);
 expect(sounds[0].pause).toHaveBeenCalled();
 expect(sent.some(m=>m.id==='t1'&&m.phase==='error'&&m.detail?.includes('15 s'))).toBe(true);
 vi.useRealTimers();
});
