/**
 * Página da fonte "Áudio do BotLive" — http://127.0.0.1:PORTA/audio#token=…
 *
 * Conecta no WebSocket local como modo `audio`, reivindica a reprodução (a
 * primeira fonte conectada toca; as demais ficam em espera) e toca os eventos
 * da saída única do bot. Nenhum visual é renderizado: no OBS a página fica
 * transparente; num navegador comum ela mostra o estado para diagnóstico, e o
 * painel de bloqueio só aparece quando o navegador impede a reprodução.
 *
 * Compilado por scripts/build-overlay.cjs para src-tauri/audio.html.
 */
import {SourcePlayer, type AudioMsg} from './audioSource';

const hash = new URLSearchParams(location.hash.replace(/^#/, ''));
const token = hash.get('token') || '';
const profile = hash.get('profile') || '';
const debug = hash.get('debug') === '1';
const inObs = typeof (window as unknown as {obsstudio?: unknown}).obsstudio !== 'undefined';

const root = document.getElementById('bl-audio-root');
let claimed = false;
let attempt = 0;
let played = 0;
let socket: WebSocket | null = null;

/** Painel visível só fora do OBS (ou com debug=1) — diagnóstico honesto. */
function show(state: string, detail = '') {
  if (inObs && !debug) return;
  if (!root) return;
  let panel = root.querySelector('.bl-panel') as HTMLDivElement | null;
  if (!panel) {
    panel = document.createElement('div');
    panel.className = 'bl-panel';
    root.appendChild(panel);
  }
  const lines: Record<string, string> = {
    connecting: 'Conectando ao aplicativo…',
    claimed: `Conectada — reproduzindo os sons do BotLive (${played} reproduzidos)`,
    standby: 'Em espera — outra fonte “Áudio do BotLive” já está ativa nesta máquina. Esta fica parada até a outra fechar.',
    reconnecting: 'Sem conexão — tentando de novo…',
    error: detail || 'Endereço incompleto. Copie o endereço de novo na tela Áudio no OBS.',
  };
  panel.innerHTML = `<b>Áudio do BotLive</b><br>${lines[state] ?? state}${state === 'claimed' && detail ? `<br>${detail}` : ''}`;
}

/** Bloqueio de reprodução (navegador comum): clique libera os próximos sons. */
function showBlocked() {
  if (!root) return;
  if (root.querySelector('.bl-banner')) return;
  const banner = document.createElement('div');
  banner.className = 'bl-banner';
  banner.textContent = 'Clique para ativar o áudio desta fonte';
  banner.onclick = () => banner.remove();
  root.appendChild(banner);
}

const player = new SourcePlayer({
  profile,
  send: (msg) => {
    if (socket && socket.readyState === 1) socket.send(JSON.stringify(msg));
  },
  onBlocked: showBlocked,
});

function scheduleReconnect() {
  attempt = Math.min(attempt + 1, 4);
  setTimeout(connect, 500 * 2 ** attempt);
}

function connect() {
  if (!token || !profile) {
    show('error');
    return;
  }
  const sock = new WebSocket(`${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/`);
  socket = sock;
  show('connecting');
  sock.onopen = () => {
    attempt = 0;
    // O primeiro define o papel: somente eventos de áudio, sem canal de eventos.
    sock.send(JSON.stringify({token, mode: 'audio'}));
  };
  sock.onmessage = (ev) => {
    let msg: AudioMsg & {payload?: {reason?: string}} = {};
    try {
      msg = JSON.parse(String(ev.data));
    } catch {
      return;
    }
    if (msg.type === 'audio.claim') {
      claimed = true;
      document.title = 'Áudio do BotLive — conectada';
      show('claimed');
      return;
    }
    if (msg.type === 'audio.standby') {
      claimed = false;
      document.title = 'Áudio do BotLive — em espera';
      show('standby');
      return;
    }
    // Segurança extra: sem reivindicação, nada toca.
    if (msg.type === 'audio.play' && !claimed) return;
    const before = player.queued;
    player.handle(msg);
    if (msg.type === 'audio.play' && player.queued > before) {
      played++;
      show('claimed');
    }
  };
  sock.onclose = () => {
    claimed = false;
    document.title = 'Áudio do BotLive — reconectando';
    show('reconnecting');
    scheduleReconnect();
  };
  sock.onerror = () => {
    try {
      sock.close();
    } catch {}
  };
}

connect();
