import {describe,expect,it} from 'vitest';
import {
 ALERT_KINDS,DEFAULT_ALERTS,fillText,isAlertKind,normalizeAlerts,sampleVars,
} from './alerts';

describe('motor de alertas: normalização',()=>{
 it('sem config salva o motor cai nos padrões desligados',()=>{
  const c=normalizeAlerts(null);
  expect(c).toEqual(DEFAULT_ALERTS);
  expect(c.enabled).toBe(false);
 });

 it('mantém o que foi salvo e completa o que falta',()=>{
  const c=normalizeAlerts({enabled:true,duration:12,events:{cheer:{on:false,tag:'BITS DO CORAÇÃO'}}});
  expect(c.enabled).toBe(true);
  expect(c.duration).toBe(12);
  expect(c.events.cheer.on).toBe(false);
  expect(c.events.cheer.tag).toBe('BITS DO CORAÇÃO');
  expect(c.events.cheer.big).toBe('{{amount}} BITS');
  expect(c.events.follow.tag).toBe('SEGUIDOR');
 });

 it('limita faixas e ignora valores fora do padrão',()=>{
  const c=normalizeAlerts({
   duration:999,speed:5,marginX:-10,panelWidth:99999,tagSize:2,
   posV:'side',posH:'middle',fontBig:'Comic Sans',brandText:'x'.repeat(80),
   events:{raid:{on:'sim',big:7}},
  });
  expect(c.duration).toBe(20);
  expect(c.speed).toBe(50);
  expect(c.marginX).toBe(0);
  expect(c.panelWidth).toBe(1760);
  expect(c.tagSize).toBe(8);
  expect(c.posV).toBe('center');
  expect(c.posH).toBe('center');
  expect(c.fontBig).toBe(DEFAULT_ALERTS.fontBig);
  expect(c.brandText).toBe(DEFAULT_ALERTS.brandText);
  expect(c.events.raid.on).toBe(true);
  expect(c.events.raid.big).toBe('{{amount}} VIEWERS');
 });

 it('os sete tipos de alerta têm padrão completo',()=>{
  for(const kind of ALERT_KINDS){
   const ev=DEFAULT_ALERTS.events[kind];
   expect(ev.on,kind).toBe(true);
   expect(sampleVars(kind).name,kind).toBeTruthy();
   expect(isAlertKind(kind)).toBe(true);
  }
  expect(isAlertKind('chat')).toBe(false);
 });

 it('cor por evento: vazio = cor global, preserva a escolhida e barra lixo',()=>{
  // Padrão: nenhuma cor própria — a tarja usa o destaque global do painel.
  for(const kind of ALERT_KINDS) expect(DEFAULT_ALERTS.events[kind].color).toBe('');
  const c=normalizeAlerts({events:{follow:{color:'#ff3b30'},raid:{color:'x'.repeat(40)}}});
  expect(c.events.follow.color).toBe('#ff3b30');
  expect(c.events.raid.color).toBe(DEFAULT_ALERTS.events.raid.color);
  expect(c.events.cheer.color).toBe('');
 });
});

describe('motor de alertas: placeholders',()=>{
 it('preenche chaves com e sem espaços e ignora desconhecidas',()=>{
  expect(fillText('{{user}} ganhou {{amount}} subs de {{sender}}',{user:'Ana',amount:3,sender:'Bia'})).toBe('Ana ganhou 3 subs de Bia');
  expect(fillText('{{ user }}',{user:'Ana'})).toBe('Ana');
  expect(fillText('{{fantasma}} aqui',{user:'Ana'})).toBe('aqui');
 });

 it('colapsa espaços dobrados quando a variável vem vazia',()=>{
  expect(fillText('{{user}} — {{message}}',{user:'Ana',message:''})).toBe('Ana —');
  expect(fillText('GG de {{user}}',{user:'AnaBits',message:'x'}).length).toBeGreaterThan(0);
 });

 it('aceita apelidos como o widget de referência: name vira user, viewers vira amount',()=>{
  expect(fillText('{{user}} deu {{amount}} subs',{name:'Generoso123',count:10})).toBe('Generoso123 deu 10 subs');
  expect(fillText('{{viewer}} assiste',{name:'Ana'})).toBe('Ana assiste');
  expect(fillText('{{amount}} VIEWERS',{viewers:37})).toBe('37 VIEWERS');
 });
});
