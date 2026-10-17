import {describe,expect,it} from 'vitest';
import {
 DEFAULT_SPOTIFY_DESIGN,elapsed,hexA,normalizeNow,normalizeSpotify,spotifyShouldShow,spotifyVars,
} from './spotifyOverlay';

describe('design da música',()=>{
 it('tem exatamente as chaves que o backend valida',()=>{
  expect(Object.keys(DEFAULT_SPOTIFY_DESIGN).sort()).toEqual([
   'albumColor','artRadius','artSize','artistColor','artistSize','barBg','barColor','barHeight','barRadius',
   'bg','cover','coverBlur','coverGlow','empty','emptyArtist','emptyText','exitTransition','font','hideOnPause',
   'layout','padding','pauseDelay','position','radius','showAlbum','showArt','showProgress','showTimes',
   'titleColor','titleSize','transition','transitionMs','width',
  ]);
 });
 it('preenche o que faltar e barra o que veio fora de faixa',()=>{
  const c=normalizeSpotify({design:{layout:'minimal',cover:'circle',position:'center',pauseDelay:30,coverGlow:true,coverBlur:true,hideOnPause:false,emptyArtist:'Comece a tocar',exitTransition:'slide'}});
  expect(c.design.layout).toBe('minimal');
  expect(c.design.cover).toBe('circle');
  expect(c.design.position).toBe('center');
  expect(c.design.pauseDelay).toBe(30);
  expect(c.design.exitTransition).toBe('slide');
  expect(c.design.emptyArtist).toBe('Comece a tocar');
  const bad=normalizeSpotify({design:{cover:'vinil',position:'meio',exitTransition:'girar',pauseDelay:45,layout:'gigante'}});
  expect(bad.design.cover).toBe('square');
  expect(bad.design.position).toBe('bottom-left');
  expect(bad.design.exitTransition).toBe('fade');
  expect(bad.design.pauseDelay, 'espera corta no teto de 30 s').toBe(30);
  expect(bad.design.layout).toBe('card');
  const fromOld=normalizeSpotify({design:{layout:'compact',emptyText:'Silêncio'}});
  expect(fromOld.design.cover, 'config salva na Fase A ganha os novos campos').toBe('square');
  expect(fromOld.design.emptyText).toBe('Silêncio');
 });
});

describe('regra de aparecer no OBS',()=>{
 const d=DEFAULT_SPOTIFY_DESIGN;
 const track={id:'1',name:'Faixa',artist:'A',album:'B',art:'',url:'',durationMs:200000,progressMs:0,timestamp:0,playing:true};
 it('tocando sempre mostra',()=>{
  expect(spotifyShouldShow(d,track,0)).toBe(true);
 });
 it('na pausa some depois da espera, 0 = na hora',()=>{
  const paused={...track,playing:false};
  expect(spotifyShouldShow(d,paused,0), 'padrão: pausa some na hora, como na Fase A').toBe(false);
  const late={...d,pauseDelay:10};
  expect(spotifyShouldShow(late,paused,9000), 'dentro da espera continua').toBe(true);
  expect(spotifyShouldShow(late,paused,10000), 'passou a espera, some').toBe(false);
  const stays={...d,hideOnPause:false};
  expect(spotifyShouldShow(stays,paused,99999), 'sem esconder na pausa, fica').toBe(true);
 });
 it('sem faixa segue o comportamento escolhido para o parado',()=>{
  expect(spotifyShouldShow(d,null,0), 'padrão some').toBe(false);
  const msg={...d,empty:'message' as const};
  expect(spotifyShouldShow(msg,null,0), 'mensagem fica visível').toBe(true);
 });
});

describe('relógio e variáveis',()=>{
 it('a posição congela na pausa e cresce tocando',()=>{
  const t={id:'1',name:'F',artist:'A',album:'B',art:'',url:'',durationMs:100000,progressMs:40000,timestamp:1000,playing:false};
  expect(elapsed(t,999999)).toBe(40000);
  expect(elapsed({...t,playing:true},1000)).toBe(40000);
  expect(elapsed({...t,playing:true},6000)).toBe(45000);
  expect(elapsed({...t,playing:true},999999), 'não passa da duração').toBe(100000);
 });
 it('as variáveis CSS carregam o design inteiro',()=>{
  const v=spotifyVars(DEFAULT_SPOTIFY_DESIGN);
  expect(v['--sp-bg']).toBe('#0f1319');
  expect(v['--sp-bar']).toBe('#1ed760');
  expect(v['--sp-w']).toBe('460px');
  expect(v['--sp-font']).toContain('"Segoe UI"');
  expect(v['--sp-glow'], 'brilho usa a cor de destaque com alfa').toBe('rgba(30,215,96,0.5)');
  const circle=spotifyVars({...DEFAULT_SPOTIFY_DESIGN,cover:'circle'});
  expect(circle['--sp-artr'], 'capa redonda dispensa o raio').toBe('999px');
 });
 it('hexA entende3 e6 dígitos e desiste do que não é cor',()=>{
  expect(hexA('#ff0000',.5)).toBe('rgba(255,0,0,0.5)');
  expect(hexA('#f00',.5)).toBe('rgba(255,0,0,0.5)');
  expect(hexA('verde',.5)).toBe('rgba(255,255,255,0.5)');
 });
});

describe('faixa transmitida',()=>{
 it('aceita o corpo do evento e o corpo snake_case do banco',()=>{
  const evt=normalizeNow({profileId:'p',playing:true,id:'t1',name:'Tocando Agora',artist:'Cactus',album:'Ao Vivo',art:'https://i.scdn.co/a.jpg',url:'https://open.spotify.com/track/t1',durationMs:201000,progressMs:83250,timestamp:1760000000000});
  expect(evt?.name).toBe('Tocando Agora');
  expect(evt?.progressMs).toBe(83250);
  const db=normalizeNow({name:'Outra',duration_ms:1000,progress_ms:10,playing:false});
  expect(db?.playing).toBe(false);
  expect(normalizeNow({profileId:'p',playing:false}), 'sem nome é nada tocando').toBe(null);
 });
});
