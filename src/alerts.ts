/**
 * Motor de alertas do BotLive — compartilhado entre a fonte do OBS
 * (src/overlayApp.ts) e a prévia do Estúdio de Alertas (src/Alertas.tsx).
 * O backend só transmite {kind, vars}; textos, cores e a coreografia saem da
 * configuração salva no perfil, então o que o streamer testa no Estúdio é o
 * mesmo código que o OBS roda. O visual segue a "janela de sistema" do widget
 * de referência: barra de status, etiqueta, nome, linha grande, linha final
 * datilografada e barra de tempo em blocos.
 */
import {ensureFont} from './overlayDesign';

export const ALERT_KINDS = ['follow', 'subscription', 'resub', 'gift', 'giftbulk', 'cheer', 'raid'] as const;
export type AlertKind = (typeof ALERT_KINDS)[number];

export const ALERT_LABELS: Record<AlertKind, string> = {
  follow: 'Seguidor',
  subscription: 'Novo sub',
  resub: 'Renovação',
  gift: 'Sub de presente',
  giftbulk: 'Presentes em lote',
  cheer: 'Bits',
  raid: 'Raid',
};

export type AlertVars = {
  name?: string;
  amount?: number | string;
  months?: number | string;
  tier?: string;
  message?: string;
  platform?: string;
};

export type AlertEventCfg = {on: boolean; tag: string; name: string; big: string; foot: string; sound: string};

export type AlertsConfig = {
  enabled: boolean;
  duration: number;
  speed: number;
  volume: number;
  posV: 'top' | 'center' | 'bottom';
  posH: 'left' | 'center' | 'right';
  marginX: number;
  marginY: number;
  panelWidth: number;
  accent: string;
  text: string;
  bg: string;
  line: string;
  fontSmall: string;
  fontBig: string;
  tagSize: number;
  nameSize: number;
  bigSize: number;
  footSize: number;
  brandText: string;
  showScanlines: boolean;
  showCorners: boolean;
  showParticles: boolean;
  showGlitch: boolean;
  events: Record<AlertKind, AlertEventCfg>;
};

const EVENT_DEFAULTS: Record<AlertKind, AlertEventCfg> = {
  follow: {on: true, tag: 'SEGUIDOR', name: '{{user}}', big: 'BEM-VINDO!', foot: 'Obrigado por seguir o canal', sound: ''},
  subscription: {on: true, tag: 'NOVO SUB', name: '{{user}}', big: '{{tier}}', foot: 'Obrigado pelo apoio!', sound: ''},
  resub: {on: true, tag: 'RENOVAÇÃO', name: '{{user}}', big: '{{months}} MESES', foot: '{{message}}', sound: ''},
  gift: {on: true, tag: 'SUB DE PRESENTE', name: '{{user}}', big: 'GANHOU UM SUB', foot: 'Obrigado pelo carinho!', sound: ''},
  giftbulk: {on: true, tag: 'PRESENTES EM LOTE', name: '{{user}}', big: '{{amount}} SUBS', foot: 'Obrigado pelo carinho!', sound: ''},
  cheer: {on: true, tag: 'BITS', name: '{{user}}', big: '{{amount}} BITS', foot: '{{message}}', sound: ''},
  raid: {on: true, tag: 'RAID', name: '{{user}}', big: '{{amount}} VIEWERS', foot: 'Sejam todos bem-vindos!', sound: ''},
};

/** Padrão espelhado em alerts.rs (defaults). */
export const DEFAULT_ALERTS: AlertsConfig = {
  enabled: false,
  duration: 8,
  speed: 100,
  volume: 70,
  posV: 'center',
  posH: 'center',
  marginX: 0,
  marginY: 0,
  panelWidth: 900,
  accent: '#c6ff00',
  text: '#ffffff',
  bg: '#161616',
  line: '#3d3d3d',
  fontSmall: 'Press Start 2P',
  fontBig: 'Montserrat',
  tagSize: 12,
  nameSize: 26,
  bigSize: 34,
  footSize: 13,
  brandText: 'BOTLIVE',
  showScanlines: true,
  showCorners: true,
  showParticles: true,
  showGlitch: true,
  events: structuredClone(EVENT_DEFAULTS),
};

const clamp = (v: unknown, min: number, max: number, fallback: number): number => {
  const n = typeof v === 'number' ? v : Number(v);
  return Number.isFinite(n) ? Math.min(max, Math.max(min, n)) : fallback;
};

const str = (v: unknown, max: number, fallback: string): string =>
  typeof v === 'string' && v.length <= max ? v : fallback;

const bool = (v: unknown, fallback: boolean): boolean => (typeof v === 'boolean' ? v : fallback);

const FONT_SET = new Set([
  'Segoe UI', 'Arial', 'Georgia', 'Verdana', 'Trebuchet MS', 'Courier New',
  'Inter', 'Montserrat', 'Poppins', 'Rubik', 'Orbitron', 'Bebas Neue',
  'Press Start 2P', 'Playfair Display', 'Rajdhani', 'Quicksand',
]);

/** Aceita o JSON salvo (ou nada) e devolve uma configuração sempre utilizável. */
export function normalizeAlerts(v: unknown): AlertsConfig {
  const o = (v && typeof v === 'object' ? v : {}) as Record<string, unknown>;
  const rawEvents = (o.events && typeof o.events === 'object' ? o.events : {}) as Record<string, unknown>;
  const events = {} as Record<AlertKind, AlertEventCfg>;
  for (const kind of ALERT_KINDS) {
    const e = (rawEvents[kind] && typeof rawEvents[kind] === 'object' ? rawEvents[kind] : {}) as Record<string, unknown>;
    const d = EVENT_DEFAULTS[kind];
    events[kind] = {
      on: bool(e.on, d.on),
      tag: str(e.tag, 160, d.tag),
      name: str(e.name, 160, d.name),
      big: str(e.big, 160, d.big),
      foot: str(e.foot, 160, d.foot),
      sound: str(e.sound, 64, d.sound),
    };
  }
  const font = (v: unknown, fallback: string): string => (typeof v === 'string' && FONT_SET.has(v) ? v : fallback);
  return {
    enabled: bool(o.enabled, DEFAULT_ALERTS.enabled),
    duration: clamp(o.duration, 3, 20, DEFAULT_ALERTS.duration),
    speed: clamp(o.speed, 50, 200, DEFAULT_ALERTS.speed),
    volume: clamp(o.volume, 0, 100, DEFAULT_ALERTS.volume),
    posV: o.posV === 'top' || o.posV === 'bottom' ? o.posV : 'center',
    posH: o.posH === 'left' || o.posH === 'right' ? o.posH : 'center',
    marginX: clamp(o.marginX, 0, 400, 0),
    marginY: clamp(o.marginY, 0, 400, 0),
    panelWidth: clamp(o.panelWidth, 320, 1760, DEFAULT_ALERTS.panelWidth),
    accent: str(o.accent, 32, DEFAULT_ALERTS.accent),
    text: str(o.text, 32, DEFAULT_ALERTS.text),
    bg: str(o.bg, 32, DEFAULT_ALERTS.bg),
    line: str(o.line, 32, DEFAULT_ALERTS.line),
    fontSmall: font(o.fontSmall, DEFAULT_ALERTS.fontSmall),
    fontBig: font(o.fontBig, DEFAULT_ALERTS.fontBig),
    tagSize: clamp(o.tagSize, 8, 72, DEFAULT_ALERTS.tagSize),
    nameSize: clamp(o.nameSize, 8, 72, DEFAULT_ALERTS.nameSize),
    bigSize: clamp(o.bigSize, 8, 72, DEFAULT_ALERTS.bigSize),
    footSize: clamp(o.footSize, 8, 72, DEFAULT_ALERTS.footSize),
    brandText: str(o.brandText, 32, DEFAULT_ALERTS.brandText),
    showScanlines: bool(o.showScanlines, DEFAULT_ALERTS.showScanlines),
    showCorners: bool(o.showCorners, DEFAULT_ALERTS.showCorners),
    showParticles: bool(o.showParticles, DEFAULT_ALERTS.showParticles),
    showGlitch: bool(o.showGlitch, DEFAULT_ALERTS.showGlitch),
    events,
  };
}

export function isAlertKind(v: unknown): v is AlertKind {
  return typeof v === 'string' && (ALERT_KINDS as readonly string[]).includes(v);
}

/**
 * Preenche `{{variável}}` (mesma sintaxe dos fluxos). Chave desconhecida vira
 * vazio e os espaços dobrados colapsam — igual ao widget de referência, que
 * aceita apelidos: user/username/viewer valem name, count vale amount.
 */
export function fillText(tpl: string, vars: Record<string, unknown>): string {
  const raw: Record<string, string> = {};
  for (const [k, v] of Object.entries(vars)) {
    if (v === undefined || v === null) continue;
    raw[k.toLowerCase()] = String(v);
  }
  const flat: Record<string, string> = {...raw};
  const aliases: Record<string, string[]> = {
    user: ['user', 'name', 'username', 'viewer'],
    amount: ['amount', 'count', 'bits', 'viewers'],
    months: ['months'],
    tier: ['tier'],
    message: ['message'],
    sender: ['sender'],
    platform: ['platform'],
  };
  for (const [target, sources] of Object.entries(aliases)) {
    const names = [target, ...sources];
    const found = names.map((s) => raw[s]).find((v) => v !== undefined);
    if (found !== undefined) for (const n of names) flat[n] = found;
  }
  const out = tpl.replace(/\{\{\s*([a-zA-Z]+)\s*\}\}/g, (_m, key: string) => flat[key.toLowerCase()] ?? '');
  return out.replace(/\s+/g, ' ').trim();
}

/** Payload de exemplo da prévia — espelha fake_vars em alerts.rs. */
export function sampleVars(kind: AlertKind): AlertVars {
  switch (kind) {
    case 'follow':
      return {name: 'NovoSeguidor', amount: 1, months: 1, tier: 'TIER 1', message: ''};
    case 'subscription':
      return {name: 'UsuarioSub', amount: 1, months: 1, tier: 'TIER 1', message: ''};
    case 'resub':
      return {name: 'UsuarioFiel', amount: 1, months: 8, tier: 'TIER 1', message: 'Melhor stream!'};
    case 'gift':
      return {name: 'Presenteado123', amount: 1, months: 1, tier: 'TIER 1', message: ''};
    case 'giftbulk':
      return {name: 'Generoso123', amount: 10, months: 1, tier: 'TIER 1', message: ''};
    case 'cheer':
      return {name: 'AnaBits', amount: 500, months: 1, tier: 'TIER 1', message: 'GG, que jogo!'};
    case 'raid':
      return {name: 'RaidMan', amount: 37, months: 1, tier: 'TIER 1', message: ''};
  }
}

/** Endereço de um som servido pela API local (rota /asset com a chave do overlay). */
export function alertSoundUrl(port: number, token: string, id: string): string {
  return `http://127.0.0.1:${port}/asset?token=${encodeURIComponent(token)}&id=${encodeURIComponent(id)}`;
}

// ---------------------------------------------------------------- DOM ------

const BAR_BLOCKS = 32;

function style(cfg: AlertsConfig): Record<string, string> {
  return {
    '--al-accent': cfg.accent,
    '--al-text': cfg.text,
    '--al-bg': cfg.bg,
    '--al-line': cfg.line,
    '--al-w': `${cfg.panelWidth}px`,
    '--al-fs-tag': `${cfg.tagSize}px`,
    '--al-fs-name': `${cfg.nameSize}px`,
    '--al-fs-big': `${cfg.bigSize}px`,
    '--al-fs-foot': `${cfg.footSize}px`,
    '--al-font-sm': `'${cfg.fontSmall}', 'Courier New', monospace`,
    '--al-font-big': `'${cfg.fontBig}', 'Segoe UI', sans-serif`,
  };
}

let styled = false;

/** Injeta o CSS do painel uma única vez (no OBS e dentro do Estúdio). */
export function ensureAlertsStyle(): void {
  if (styled || typeof document === 'undefined') return;
  styled = true;
  const el = document.createElement('style');
  el.id = 'bl-alerts-style';
  el.textContent = CSS;
  document.head.appendChild(el);
  ensureFont('Press Start 2P');
  ensureFont('Montserrat');
}

/** Painel no estado final — usado na prévia parada e como base da animação. */
export function buildPanel(cfg: AlertsConfig, kind: AlertKind, vars: AlertVars): HTMLElement {
  const ev = cfg.events[kind];
  const panel = document.createElement('div');
  panel.className = 'bl-alert';
  for (const [k, v] of Object.entries(style(cfg))) panel.style.setProperty(k, v);
  if (cfg.showCorners) {
    for (const corner of ['tl', 'tr', 'bl', 'br']) {
      const c = document.createElement('span');
      c.className = `al-corner ${corner}`;
      panel.appendChild(c);
    }
  }
  if (cfg.showScanlines) panel.appendChild(Object.assign(document.createElement('span'), {className: 'al-scan'}));
  const head = document.createElement('header');
  head.className = 'al-head';
  const brand = document.createElement('span');
  brand.className = 'al-brand';
  brand.textContent = cfg.brandText;
  const type = document.createElement('span');
  type.className = 'al-type';
  type.textContent = ALERT_LABELS[kind];
  const plat = document.createElement('span');
  plat.className = 'al-plat';
  plat.textContent = (vars.platform || 'twitch').toUpperCase();
  const status = document.createElement('span');
  status.className = 'al-status';
  status.innerHTML = '<i></i><b>OK</b>';
  head.append(brand, type, plat, status);
  panel.appendChild(head);
  const lines: Record<'tag' | 'name' | 'big' | 'foot', HTMLElement> = {
    tag: document.createElement('div'),
    name: document.createElement('div'),
    big: document.createElement('div'),
    foot: document.createElement('div'),
  };
  lines.tag.className = 'al-tag';
  lines.name.className = 'al-name';
  lines.big.className = 'al-big';
  lines.foot.className = 'al-foot';
  lines.tag.textContent = fillText(ev.tag, vars as Record<string, unknown>);
  lines.name.textContent = fillText(ev.name, vars as Record<string, unknown>);
  lines.big.textContent = fillText(ev.big, vars as Record<string, unknown>);
  lines.foot.textContent = fillText(ev.foot, vars as Record<string, unknown>);
  panel.append(lines.tag, lines.name, lines.big, lines.foot);
  const time = document.createElement('div');
  time.className = 'al-time';
  for (let i = 0; i < BAR_BLOCKS; i++) time.appendChild(document.createElement('i'));
  panel.appendChild(time);
  return panel;
}

/** Posiciona o painel dentro do palco (mesmo flex do widget de referência). */
function placeStage(host: HTMLElement, cfg: AlertsConfig): void {
  host.classList.add('bl-ov-alerts');
  host.style.setProperty('justify-content', cfg.posV === 'top' ? 'flex-start' : cfg.posV === 'bottom' ? 'flex-end' : 'center');
  host.style.setProperty('align-items', cfg.posH === 'left' ? 'flex-start' : cfg.posH === 'right' ? 'flex-end' : 'center');
  host.style.setProperty('padding', `${cfg.marginY}px ${cfg.marginX}px`);
}

/** Prévia parada: o painel já nasce visível, com barra cheia. */
export function renderStatic(host: HTMLElement, cfg: AlertsConfig, kind: AlertKind, vars: AlertVars): void {
  ensureAlertsStyle();
  placeStage(host, cfg);
  const panel = buildPanel(cfg, kind, vars);
  panel.classList.add('is-static');
  host.replaceChildren(panel);
  void host.offsetWidth;
}

const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));
const SCRAMBLE = '▚▞█▙▟/\\|<>+=·01';

/** Diminui a fonte da linha grande até caber no painel. */
function fit(panel: HTMLElement): void {
  const big = panel.querySelector('.al-big') as HTMLElement | null;
  if (!big) return;
  let size = parseFloat(getComputedStyle(big).fontSize);
  while (big.scrollWidth > panel.clientWidth - 28 && size > 12) {
    size -= 2;
    big.style.fontSize = `${size}px`;
  }
}

function burst(panel: HTMLElement, cfg: AlertsConfig): void {
  const fx = panel.querySelector('.al-fx') as HTMLElement | null;
  if (!fx) return;
  const r = panel.getBoundingClientRect();
  for (let i = 0; i < 24; i++) {
    const px = document.createElement('span');
    px.className = 'al-px';
    const ang = Math.random() * Math.PI * 2;
    const dist = 60 + Math.random() * 160;
    px.style.left = `${r.width / 2}px`;
    px.style.top = `${r.height / 2}px`;
    px.style.setProperty('--dx', `${Math.cos(ang) * dist}px`);
    px.style.setProperty('--dy', `${Math.sin(ang) * dist}px`);
    fx.appendChild(px);
    setTimeout(() => px.remove(), 1200);
  }
}

export type AlertPlayback = {cfg: AlertsConfig; kind: AlertKind; vars: AlertVars; audio?: {src: string; volume: number} | null};

/**
 * Toca um alerta no palco: entrada coreografada, barra de tempo correndo por
 * `duration` segundos reais e saída em cascata. Resolve quando o painel some —
 * o chamador costuma encadear pela fila (queueAlert).
 */
export async function runAlert(host: HTMLElement, alert: AlertPlayback): Promise<void> {
  const {cfg, kind, vars} = alert;
  ensureAlertsStyle();
  placeStage(host, cfg);
  const k = 100 / cfg.speed;
  const t = (ms: number) => ms * k;
  const panel = buildPanel(cfg, kind, vars);
  const fx = document.createElement('div');
  fx.className = 'al-fx';
  panel.appendChild(fx);
  host.replaceChildren(panel);
  const q = (sel: string) => panel.querySelector(sel) as HTMLElement;
  const tagEl = q('.al-tag');
  const nameEl = q('.al-name');
  const bigEl = q('.al-big');
  const footEl = q('.al-foot');
  const statusEl = q('.al-status b');
  const blocks = [...panel.querySelectorAll('.al-time i')] as HTMLElement[];

  const finalTag = tagEl.textContent || '';
  tagEl.textContent = '';
  const finalFoot = footEl.textContent || '';
  footEl.textContent = '';
  nameEl.classList.add('al-start-hidden');
  bigEl.classList.add('al-start-hidden');

  panel.classList.add('al-booting');
  void panel.offsetWidth;
  await sleep(t(260));
  panel.classList.remove('al-booting');

  if (alert.audio) {
    const a = new Audio(alert.audio.src);
    a.volume = Math.min(1, Math.max(0, alert.audio.volume / 100));
    setTimeout(() => void a.play().catch(() => {}), t(120));
  }

  // Etiqueta "decodificando": caracteres aleatórios resolvem da esquerda.
  for (let i = 0; i <= finalTag.length; i++) {
    const noise = finalTag
      .slice(i)
      .split('')
      .map((c) => (c === ' ' ? ' ' : SCRAMBLE[Math.floor(Math.random() * SCRAMBLE.length)]))
      .join('');
    tagEl.textContent = finalTag.slice(0, i) + noise;
    await sleep(t(45));
  }
  tagEl.textContent = finalTag;

  nameEl.classList.remove('al-start-hidden');
  nameEl.classList.add('al-reveal', 'al-slidein');
  await sleep(t(300));

  bigEl.classList.remove('al-start-hidden');
  bigEl.classList.add('al-reveal', 'al-slam');
  fit(panel);
  await sleep(t(320));
  panel.classList.add('al-shake');
  setTimeout(() => panel.classList.remove('al-shake'), t(400));
  if (cfg.showGlitch) {
    panel.classList.add('al-glitching');
    setTimeout(() => panel.classList.remove('al-glitching'), t(900));
  }
  if (cfg.showParticles) burst(panel, cfg);
  statusEl.textContent = 'OK';

  for (let i = 0; i <= finalFoot.length; i++) {
    footEl.textContent = finalFoot.slice(0, i);
    await sleep(t(16));
  }

  // Barra de tempo: acende em sequência durante a duração e esvazia no fim.
  const barMs = Math.max(3000, cfg.duration * 1000);
  blocks.forEach((b, i) => {
    b.style.animationDelay = `${(barMs * i) / BAR_BLOCKS}ms`;
    b.classList.add('on');
  });
  await sleep(barMs);
  blocks.forEach((b, i) => {
    b.classList.remove('on');
    b.style.animationDelay = `${(barMs * 0.2 * i) / BAR_BLOCKS}ms`;
  });
  await sleep(barMs * 0.2 + t(300));

  statusEl.textContent = 'ENCERRANDO…';
  for (const el of [tagEl, nameEl, bigEl, footEl]) {
    el.classList.add('al-hiding');
    await sleep(t(90));
  }
  panel.classList.add('al-unbooting');
  await sleep(t(360));
  panel.remove();
  if (!host.childElementCount) host.replaceChildren();
}

const chains = new WeakMap<HTMLElement, Promise<void>>();

/** Enfileira alertas por palco: nunca dois painéis ao mesmo tempo. */
export function queueAlert(host: HTMLElement, alert: AlertPlayback): Promise<void> {
  const prev = chains.get(host) ?? Promise.resolve();
  const next = prev.then(() => runAlert(host, alert)).catch(() => {});
  chains.set(host, next);
  return next;
}

const CSS = `
.bl-ov-alerts{position:absolute;inset:0;display:flex;pointer-events:none;overflow:hidden}
.bl-alert{position:relative;width:var(--al-w);max-width:calc(100% - 32px);background:var(--al-bg);border:2px solid var(--al-line);color:var(--al-text);font-family:var(--al-font-sm);padding:14px 20px 12px;display:flex;flex-direction:column;gap:10px;box-shadow:0 0 0 2px rgba(0,0,0,.55),0 12px 40px rgba(0,0,0,.45)}
.bl-alert.al-booting{animation:al-boot .26s steps(6) both}
.bl-alert.al-unbooting{animation:al-unboot .36s steps(6) both}
.al-corner{position:absolute;width:11px;height:11px;border:2px solid var(--al-accent)}
.al-corner.tl{top:-2px;left:-2px;border-right:0;border-bottom:0}
.al-corner.tr{top:-2px;right:-2px;border-left:0;border-bottom:0}
.al-corner.bl{bottom:-2px;left:-2px;border-right:0;border-top:0}
.al-corner.br{bottom:-2px;right:-2px;border-left:0;border-top:0}
.al-scan{position:absolute;inset:0;pointer-events:none;background:repeating-linear-gradient(0deg,transparent 0 3px,rgba(0,0,0,.22) 3px 4px)}
.al-head{display:flex;align-items:center;gap:10px;border-bottom:1px solid var(--al-line);padding-bottom:8px;font-size:10px;letter-spacing:1px;text-transform:uppercase;color:var(--al-accent)}
.al-head .al-brand{color:var(--al-text);opacity:.7}
.al-head .al-plat{opacity:.45}
.al-status{margin-left:auto;display:flex;align-items:center;gap:6px}
.al-status i{width:7px;height:7px;background:var(--al-accent);animation:al-blink 1s steps(2,end) infinite}
.al-status b{font-weight:400}
.al-tag{display:inline-block;align-self:flex-start;background:var(--al-accent);color:#0f0f0f;padding:4px 9px;font-size:var(--al-fs-tag)}
.al-name{font-family:var(--al-font-big);font-size:var(--al-fs-name);font-weight:700;color:var(--al-accent);overflow-wrap:anywhere}
.al-big{font-family:var(--al-font-big);font-size:var(--al-fs-big);font-weight:900;line-height:1.08;text-transform:uppercase;overflow-wrap:anywhere}
.al-foot{font-size:var(--al-fs-foot);opacity:.82;min-height:1.2em;overflow-wrap:anywhere;white-space:pre-wrap}
.al-start-hidden{visibility:hidden}
.al-reveal{animation:al-reveal .3s steps(8) both}
.al-slidein{animation:al-slidein .3s steps(6) both}
.al-slam{animation:al-slam .3s steps(4) both}
.al-hiding{animation:al-hide .25s steps(6) both}
.al-shake{animation:al-shake .4s steps(3) 1}
.bl-alert.al-glitching .al-big,.bl-alert.al-glitching .al-name{animation:al-glitch .3s steps(2) 3}
.al-time{display:flex;gap:2px;height:6px;margin-top:2px}
.al-time i{flex:1;background:rgba(255,255,255,.07)}
.al-time i.on{background:var(--al-accent);animation:al-on .18s steps(2) both}
.al-fx{position:absolute;inset:0;pointer-events:none;z-index:5;overflow:visible}
.al-px{position:absolute;width:6px;height:6px;background:var(--al-accent);animation:al-px .9s steps(8) both}
@keyframes al-boot{from{clip-path:inset(0 0 100% 0)}to{clip-path:inset(0 0 0 0)}}
@keyframes al-unboot{from{clip-path:inset(0 0 0 0);opacity:1}to{clip-path:inset(48% 0 48% 0);opacity:0}}
@keyframes al-reveal{from{clip-path:inset(0 100% 0 0)}to{clip-path:inset(0 0 0 0)}}
@keyframes al-hide{from{clip-path:inset(0 0 0 0)}to{clip-path:inset(0 0 0 100%)}}
@keyframes al-slidein{from{transform:translateX(-28px)}to{transform:translateX(0)}}
@keyframes al-slam{0%{transform:scale(1.5)}60%{transform:scale(.96)}100%{transform:scale(1)}}
@keyframes al-shake{0%,100%{transform:translate(0,0)}25%{transform:translate(-4px,2px)}50%{transform:translate(4px,-2px)}75%{transform:translate(-2px,-2px)}}
@keyframes al-glitch{0%,100%{text-shadow:none;transform:translate(0,0)}25%{text-shadow:-3px 0 #ff3b6b,3px 0 #35c9ff;transform:translate(2px,-1px)}60%{text-shadow:3px 0 #ff3b6b,-3px 0 #35c9ff;transform:translate(-2px,1px)}}
@keyframes al-blink{50%{opacity:.15}}
@keyframes al-on{from{opacity:.12}to{opacity:1}}
@keyframes al-px{from{transform:translate(0,0);opacity:1}to{transform:translate(var(--dx),var(--dy));opacity:0}}
.bl-alert.is-static .al-time i{background:var(--al-accent);opacity:1}
`;
