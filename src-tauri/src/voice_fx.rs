//! Voz da live: captura o microfone, aplica efeito e juta num dispositivo
//! virtual (VB-Cable) que o OBS captura. Etapa 1 = passa-água (sem efeito),
//! para provar captura + roteamento com latência baixa antes de os efeitos.
//! Nada aqui toca no "Ouvir minha voz", no microfone do Windows nem no OBS.

use crate::engine::Runtime;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub const KIND: &str = "voiceFx";
pub const OFF: &str = "off";
pub const STARTING: &str = "starting";
pub const RUNNING: &str = "running";
pub const FAILED: &str = "failed";

/// Taxa única para entrada e saída: WASAPI compartilhado aceita 48 kHz em tudo
/// e evita voz no tom errado por causa de taxas diferentes entre os aparelhos.
const RATE: u32 = 48_000;
/// Anel folgado para absorver variação de tamanho de buffer entre os dois lados.
const RING: usize = 8192;
/// Corte de ruído inicial: -45 dBFS tira ventilador/teclado e preserva voz baixa.
const DEFAULT_GATE_DB: f32 = -45.0;
const NO_OUTPUT: &str = "Selecione a entrada do VB-Cable (CABLE Input) como saída. Instale o VB-Cable se ainda não tiver.";

/// Vozes aceitas pelo resgate e pela tela. "normal" volta ao Pass (sem efeito).
pub const EFFECTS: &[&str] = &["normal", "fina", "grossa", "robo", "engracada", "eco"];

#[derive(Clone, Serialize)]
pub struct Status {
 pub phase: String,
 pub message: String,
 pub input: String,
 pub output: String,
 pub effect: String,
 pub gate_db: f32,
}

impl Default for Status {
 fn default() -> Self {
  Self { phase: OFF.into(), message: "Voz ao vivo parada.".into(), input: String::new(), output: String::new(), effect: "normal".into(), gate_db: DEFAULT_GATE_DB }
 }
}

enum Cmd {
 Start { input: String, output: String },
 Stop,
}

/// Estado global da voz ao live nesta máquina. Seguro entre threads: os Streams
/// (que não cruzam de thread) vivem presos na thread do motor, nunca aqui.
pub struct Fx {
 state: Arc<Mutex<Status>>,
 engine: Mutex<Option<Sender<Cmd>>>,
 level: Arc<AtomicU32>,
 proc: Arc<Mutex<Processor>>,
 /// Número da última troca de voz. Um temporizador só reverte se ainda for o
 /// mais novo, para resgates sobrepostos não se apagarem.
 gen: Arc<AtomicU64>,
 /// Corte de ruído de fundo em dBFS (guardado como bits de f32; o callback de
 /// saída relê o valor a cada bloco, então vale na hora).
 gate_db: Arc<AtomicU32>,
}

impl Default for Fx {
 fn default() -> Self {
  Self { state: Arc::new(Mutex::new(Status::default())), engine: Mutex::new(None), level: Arc::new(AtomicU32::new(0)), proc: Arc::new(Mutex::new(Processor::default())), gen: Arc::new(AtomicU64::new(0)), gate_db: Arc::new(AtomicU32::new(f32::to_bits(DEFAULT_GATE_DB))) }
 }
}

impl Fx {
 fn publish(&self, rt: &Runtime) {
  let s = self.state.lock().unwrap().clone();
  rt.emit(KIND, serde_json::to_value(&s).unwrap_or(Value::Null));
 }
 fn set(&self, phase: &str, message: &str, input: &str, output: &str) {
  let mut s = self.state.lock().unwrap();
  s.phase = phase.into();
  s.message = message.into();
  s.input = input.into();
  s.output = output.into();
 }
 pub fn get(&self) -> Value {
  serde_json::to_value(self.state.lock().unwrap().clone()).unwrap_or(Value::Null)
 }
 pub fn level(&self) -> Value {
  json!({ "level": f32::from_bits(self.level.load(Ordering::Relaxed)) })
 }
 pub fn save_selection(&self, rt: &Runtime, p: &str, input: &str, output: &str) -> Result<Value, String> {
  rt.db.profile(p)?;
  // Junta, nunca sobrescreve: o corte de ruído guardado sobrevive à troca de aparelhos.
  let mut m = rt.db.module(p, "voiceFx");
  if !m.is_object() {
   m = json!({});
  }
  m["input"] = json!(input);
  m["output"] = json!(output);
  m["gate_db"] = json!(self.state.lock().unwrap().gate_db);
  rt.db.set_module(p, "voiceFx", &m)?;
  Ok(json!({ "input": input, "output": output, "gateDb": self.state.lock().unwrap().gate_db }))
 }
 pub fn load_selection(&self, rt: &Runtime, p: &str) -> Result<Value, String> {
  rt.db.profile(p)?;
  Ok(rt.db.module(p, "voiceFx"))
 }
 pub fn start(&self, rt: &Arc<Runtime>, p: &str, input: &str, output: &str) -> Result<Value, String> {
  rt.db.profile(p)?;
  if input.trim().is_empty() || output.trim().is_empty() {
   return Err(NO_OUTPUT.into());
  }
  let mut slot = self.engine.lock().unwrap();
  if slot.is_none() {
   let (tx, rx) = channel::<Cmd>();
   let rt2 = rt.clone();
   let st = self.state.clone();
   let lvl = self.level.clone();
   let prc = self.proc.clone();
   let gdb = self.gate_db.clone();
   std::thread::spawn(move || engine_loop(rt2, rx, st, lvl, prc, gdb));
   *slot = Some(tx);
  }
  let tx = slot.as_ref().ok_or("Motor de áudio indisponível.")?.clone();
  drop(slot);
  // O corte de ruído guardado passa a valer nesta sessão.
  let gate = rt.db.module(p, "voiceFx").get("gate_db").and_then(Value::as_f64).map(|d| d as f32).unwrap_or(DEFAULT_GATE_DB);
  self.gate_db.store(f32::to_bits(gate), Ordering::Relaxed);
  self.state.lock().unwrap().gate_db = gate;
  self.set(STARTING, "Iniciando a voz ao vivo…", input, output);
  self.publish(rt);
  tx.send(Cmd::Start { input: input.into(), output: output.into() })
   .map_err(|_| "Motor de áudio indisponível.")?;
  let _ = self.save_selection(rt, p, input, output);
  Ok(json!({ "phase": STARTING }))
 }
 pub fn stop(&self, rt: &Arc<Runtime>, p: &str) -> Result<Value, String> {
  rt.db.profile(p)?;
  let slot = self.engine.lock().unwrap();
  if let Some(tx) = slot.as_ref() {
   let _ = tx.send(Cmd::Stop);
  }
  Ok(Value::Null)
 }
 /// Troca a voz ao vivo para um efeito pronto (fina, grossa, robo/engracada, eco…).
 /// Devolve o número da troca, que um efeito temporário usa para só reverter
 /// se ninguém tiver pedido outra voz no meio-tempo.
 pub fn set_effect(&self, rt: &Arc<Runtime>, p: &str, name: &str) -> Result<Value, String> {
  rt.db.profile(p)?;
  let e = effect_for(name);
  let gen = self.gen.fetch_add(1, Ordering::SeqCst) + 1;
  self.proc.lock().unwrap().set_effect(e);
  {
   let mut s = self.state.lock().unwrap();
   s.effect = s_effect_name(&e).into();
  }
  self.publish(rt);
  Ok(json!({ "effect": s_effect_name(&e), "gen": gen }))
 }
 /// A voz ao vivo está com áudio de fato circulando?
 pub fn is_running(&self) -> bool {
  self.state.lock().unwrap().phase == RUNNING
 }
 /// Volta para a voz normal, mas só se a troca `gen` ainda for a mais recente.
 pub fn revert_if(&self, rt: &Arc<Runtime>, p: &str, gen: u64) {
  if self.gen.load(Ordering::SeqCst) != gen {
   return; // já mudou depois; deixa a voz nova intacta
  }
  let _ = self.set_effect(rt, p, "normal");
 }
 /// Ajusta o corte de ruído de fundo (-80 a -20 dBFS; -80 quase não corta, -20
 /// corta quase tudo). Vale na hora, mesmo com a voz ao vivo rodando.
 pub fn set_gate(&self, rt: &Runtime, p: &str, db: f32) -> Result<Value, String> {
  rt.db.profile(p)?;
  if !(-80.0..=-20.0).contains(&db) {
   return Err("O corte de ruído vai de -80 (quase nada corta) a -20 dB.".into());
  }
  self.gate_db.store(f32::to_bits(db), Ordering::Relaxed);
  let (input, output) = { let mut s = self.state.lock().unwrap(); s.gate_db = db; (s.input.clone(), s.output.clone()) };
  let _ = self.save_selection(rt, p, &input, &output);
  self.publish(rt);
  Ok(json!({ "gateDb": db }))
 }
}

fn s_effect_name(e: &Effect) -> &'static str {
 match e {
  Effect::Pass => "normal",
  Effect::Pitch(r) if *r > 1.0 => "fina",
  Effect::Pitch(_) => "grossa",
  Effect::Ring(..) => "robo",
  Effect::Wobble(..) => "engracada",
  Effect::Echo(..) => "eco",
 }
}

/// Thread única que abriga os Streams. Roda o loop de comandos; ao receber
/// Start reergue os fluxos, ao Stop derruba. Streams nunca saem daqui.
fn engine_loop(rt: Arc<Runtime>, rx: Receiver<Cmd>, state: Arc<Mutex<Status>>, level: Arc<AtomicU32>, proc: Arc<Mutex<Processor>>, gate_db: Arc<AtomicU32>) {
 let mut streams: Vec<cpal::Stream> = Vec::new();
 loop {
  match rx.recv() {
   Ok(Cmd::Start { input, output }) => {
    streams.clear();
    set_publish(&state, &rt, STARTING, "Iniciando a voz ao vivo…", &input, &output);
    match build(&rt, &state, &input, &output, &level, &proc, &gate_db) {
     Ok(v) => {
      streams = v;
      set_publish(&state, &rt, RUNNING, "Voz ao vivo ativa. Fale para ouvir no OBS.", &input, &output);
     }
     Err(e) => {
      set_publish(&state, &rt, FAILED, &e, &input, &output);
      rt.log("", KIND, &e, "error");
     }
    }
   }
   Ok(Cmd::Stop) => {
    streams.clear();
    let (input, output) = { let s = state.lock().unwrap(); (s.input.clone(), s.output.clone()) };
    set_publish(&state, &rt, OFF, "Voz ao vivo parada.", &input, &output);
   }
   Err(_) => break,
  }
 }
}

fn set_publish(state: &Arc<Mutex<Status>>, rt: &Runtime, phase: &str, message: &str, input: &str, output: &str) {
 { let mut s = state.lock().unwrap(); s.phase = phase.into(); s.message = message.into(); s.input = input.into(); s.output = output.into(); }
 rt.emit(KIND, serde_json::to_value(state.lock().unwrap().clone()).unwrap_or(Value::Null));
}

/// Reamostra entre taxas diferentes com interpolação linear simples. O pipeline
/// interno é sempre 48 kHz; só a entrada (microfone) e a saída convertem, para
/// qualquer aparelho abrir sem depender da taxa nativa dele.
struct Resample { step: f64, pos: f64, hold: Vec<f32>, last: f32 }
impl Resample {
 fn new(from: u32, to: u32) -> Self { Self { step: from as f64 / to as f64, pos: 0.0, hold: Vec::new(), last: 0.0 } }
 fn feed(&mut self, input: &[f32]) { if let Some(&l) = input.last() { self.last = l; } self.hold.extend_from_slice(input); }
 /// Uma amostra pronta, ou None quando ainda não há dado suficiente.
 fn one(&mut self) -> Option<f32> {
  let i = self.pos.floor() as usize;
  if i + 1 >= self.hold.len() { return None; }
  let frac = (self.pos - i as f64) as f32;
  let a = self.hold[i]; let b = self.hold[i + 1];
  let s = a + (b - a) * frac;
  self.pos += self.step;
  let k = self.pos.floor() as usize;
  if k > 0 { self.hold.drain(..k); self.pos -= k as f64; }
  Some(s)
 }
 /// Lado da entrada: converte tudo o que dá para converter.
 fn convert(&mut self, input: &[f32], out: &mut Vec<f32>) { self.feed(input); while let Some(s) = self.one() { out.push(s); } }
 /// Lado da saída: produz exatamente n amostras; se faltar dado, segura o último.
 fn pull(&mut self, n: usize, out: &mut Vec<f32>) { for _ in 0..n { out.push(self.one().unwrap_or(self.last)); } }
}

/// Porta de ruído: corta o fundo do microfone (ventilador, teclado, chiado)
/// quando ninguém fala. Envelope lento segura o fim da palavra; ataque rápido
/// e saída lenta evitam estalo. Limiar em amplitude linear (vem em dBFS).
struct Gate { thr: f32, env: f32, hold: i32, gain: f32 }
/// Caída do envelope por amostra (≈ 21 ms a 48 kHz).
const GATE_ENV: f32 = 0.999;
/// Mantém a porta aberta ~300 ms depois da fala para não cortar sílabas.
const GATE_HOLD: i32 = 14_400;
/// Ataque (~5 ms até o ganho pleno) e saída (~40 ms até o silêncio).
const GATE_ATTACK: f32 = 0.004;
const GATE_RELEASE: f32 = 0.0005;

impl Gate {
 fn new(thr: f32) -> Self { Self { thr: thr.clamp(1e-5, 1.0), env: 0.0, hold: 0, gain: 0.0 } }
 fn set_threshold(&mut self, thr: f32) { self.thr = thr.clamp(1e-5, 1.0); }
 fn process(&mut self, x: f32) -> f32 {
  let a = x.abs();
  self.env = a.max(self.env * GATE_ENV);
  if self.env >= self.thr {
   self.hold = GATE_HOLD;
  } else if self.hold > 0 {
   self.hold -= 1;
  }
  let step = if self.hold > 0 { GATE_ATTACK } else { -GATE_RELEASE };
  self.gain = (self.gain + step).clamp(0.0, 1.0);
  x * self.gain
 }
}

/// dBFS → amplitude linear: o limiar da porta é sempre falado em dB na tela.
fn db_to_amp(db: f32) -> f32 { 10f32.powf(db / 20.0) }

/// Liga microfone → anel (48 kHz) → efeito → saída. Usa a taxa nativa de cada
/// aparelho e converte só nas pontas: microfone nenhum fica de fora por taxa.
fn build(rt: &Arc<Runtime>, state: &Arc<Mutex<Status>>, input_name: &str, output_name: &str, level: &Arc<AtomicU32>, proc: &Arc<Mutex<Processor>>, gate_db: &Arc<AtomicU32>) -> Result<Vec<cpal::Stream>, String> {
 let host = cpal::default_host();
 let in_dev = find(&host, true, input_name).ok_or("Microfone não encontrado. Confira a seleção de entrada.")?;
 let out_dev = find(&host, false, output_name).ok_or(NO_OUTPUT)?;
 // Cada aparelho na taxa que ele já aceita (o default dele) — nunca força 48 kHz.
 let in_sup = in_dev.default_input_config().map_err(|_| "O microfone não expõe formato de áudio.")?;
 let out_sup = out_dev.default_output_config().map_err(|_| "O dispositivo de saída não expõe formato de áudio.")?;
 let in_ch = in_sup.channels();
 let out_ch = out_sup.channels();
 let in_rate = in_sup.sample_rate().0.max(1);
 let out_rate = out_sup.sample_rate().0.max(1);
 let in_cfg = cpal::StreamConfig { channels: in_ch, sample_rate: cpal::SampleRate(in_rate), buffer_size: cpal::BufferSize::Default };
 let out_cfg = cpal::StreamConfig { channels: out_ch, sample_rate: cpal::SampleRate(out_rate), buffer_size: cpal::BufferSize::Default };

 let (mut prod, mut cons) = ringbuf::HeapRb::<f32>::new(RING).split();

 let lvl = level.clone();
 let mut in_rs = Resample::new(in_rate, RATE);
 let mut in_mono: Vec<f32> = Vec::new();
 let input = in_dev
  .build_input_stream::<f32, _, _>(
   &in_cfg,
   move |data: &[f32], _| {
    let mono = downmix(data, in_ch);
    lvl.store(f32::to_bits(peak(&mono)), Ordering::Relaxed);
    in_mono.clear();
    in_rs.convert(&mono, &mut in_mono);
    prod.push_slice(&in_mono);
   },
   err_cb(rt.clone(), state.clone(), "microfone"),
   None,
  )
  .map_err(map_build("microfone"))?;

 let proc = proc.clone();
 let gdb = gate_db.clone();
 let mut gate = Gate::new(db_to_amp(f32::from_bits(gdb.load(Ordering::Relaxed))));
 let mut out_rs = Resample::new(RATE, out_rate);
 let mut out_mono: Vec<f32> = Vec::new();
 let output = out_dev
  .build_output_stream::<f32, _, _>(
   &out_cfg,
   move |data: &mut [f32], _| {
    let frames = data.len() / out_ch.max(1) as usize;
    // Amostras de 48 kHz que cobrem `frames` na taxa de saída (+2 para interpolar).
    let need = ((frames as f64) * (RATE as f64) / (out_rate as f64)).ceil() as usize + 2;
    let mut src = vec![0f32; need];
    let got = cons.pop_slice(&mut src);
    // Porta de ruído antes do efeito: o fundo não entra nem no eco nem no anel.
    gate.set_threshold(db_to_amp(f32::from_bits(gdb.load(Ordering::Relaxed))));
    {
     let mut fx = proc.lock().unwrap();
     for s in src[..got].iter_mut() { *s = fx.next(gate.process(*s)); }
    }
    out_rs.feed(&src[..got]);
    out_mono.clear();
    out_rs.pull(frames, &mut out_mono);
    upmix(&out_mono, out_ch, data);
   },
   err_cb(rt.clone(), state.clone(), "saída"),
   None,
  )
  .map_err(map_build("saída"))?;

 input.play().map_err(|_| "Não foi possível iniciar a captura do microfone.")?;
 output.play().map_err(|_| "Não foi possível iniciar a saída do áudio.")?;
 Ok(vec![input, output])
}

fn err_cb(rt: Arc<Runtime>, state: Arc<Mutex<Status>>, label: &'static str) -> impl FnMut(cpal::StreamError) {
 move |e| {
  let msg = format!("{label}: {e}");
  { let mut s = state.lock().unwrap(); s.phase = FAILED.into(); s.message = "A conexão de áudio caiu. Confira os dispositivos.".into(); }
  rt.emit(KIND, serde_json::to_value(state.lock().unwrap().clone()).unwrap_or(Value::Null));
  rt.log("", KIND, &msg, "error");
 }
}

fn map_build(what: &'static str) -> impl Fn(cpal::BuildStreamError) -> String {
 move |e| format!("Não foi possível abrir o {what}: {e}. Confira se o aparelho está ligado e aceita 48 kHz.")
}

fn find(host: &cpal::Host, input: bool, name: &str) -> Option<cpal::Device> {
 let it = if input { host.input_devices().ok() } else { host.output_devices().ok() };
 if let Some(devs) = it {
  for d in devs {
   if d.name().ok().as_deref() == Some(name) {
    return Some(d);
   }
  }
 }
 if input { host.default_input_device() } else { host.default_output_device() }
}

pub fn devices() -> Value {
 let host = cpal::default_host();
 json!({ "inputs": list(&host, true), "outputs": list(&host, false) })
}

fn list(host: &cpal::Host, input: bool) -> Vec<Value> {
 let (default_name, devs) = if input {
  (host.default_input_device().and_then(|d| d.name().ok()), host.input_devices().ok())
 } else {
  (host.default_output_device().and_then(|d| d.name().ok()), host.output_devices().ok())
 };
 let mut out = Vec::new();
 if let Some(it) = devs {
  for d in it {
   let name = d.name().unwrap_or_default();
   let is_def = default_name.as_deref() == Some(name.as_str());
   out.push(json!({ "name": name, "default": is_def }));
  }
 }
 out
}

/// Interleaved → mono (média dos canais).
fn downmix(data: &[f32], channels: u16) -> Vec<f32> {
 let ch = channels.max(1) as usize;
 data.chunks(ch).map(|f| f.iter().sum::<f32>() / ch as f32).collect()
}

/// Mono → interleaved (espelha o mono em todos os canais de saída).
fn upmix(mono: &[f32], channels: u16, out: &mut [f32]) {
 let ch = channels.max(1) as usize;
 for (i, &s) in mono.iter().enumerate() {
  for c in 0..ch {
   let idx = i * ch + c;
   if idx < out.len() {
    out[idx] = s;
   }
  }
 }
}

fn peak(mono: &[f32]) -> f32 {
 mono.iter().fold(0f32, |m, &v| m.max(v.abs()))
}

/// Efeito aplicado à voz ao vivo. `Pass` deixa a voz igual.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Effect {
 Pass,
 /// Afina sem mudar a velocidade: razão >1 = voz fina, <1 = voz grossa.
 Pitch(f64),
 /// Robô: multiplica a voz por uma onda (frequência Hz, profundidade 0..1).
 Ring(f64, f64),
 /// Voz engraçada: vibrato cartoon (taxa em Hz, desvio do atraso em amostras).
 Wobble(f64, f64),
 /// Eco: (atraso em ms, realimentação 0..0.9, mistura 0..1).
 Echo(f64, f64, f64),
}

/// Biblioteca embutida de vozes. O nome vira efeito pronto para o resgate.
pub fn effect_for(name: &str) -> Effect {
 match name {
  "fina" => Effect::Pitch(1.5),
  "grossa" => Effect::Pitch(0.7),
  "robo" => Effect::Ring(45.0, 1.0),
  "engracada" => Effect::Wobble(9.0, SWEEP * 0.16),
  "eco" => Effect::Echo(220.0, 0.45, 0.5),
  _ => Effect::Pass,
 }
}

/// Varredura do pitch shifter (amostras). Quanto maior, menos artefato de tremolo.
const SWEEP: f64 = 2048.0;
/// Muda de efeito com um breve fade para não estalar no ouvido.
const FADE: i32 = 240;
/// Linha de atraso compartilhada por pitch e eco (deve caber 2 varreduras + folga).
const DELAY_LINE: usize = 8192;

/// Estado de áudio do efeito. Vive travado na thread de áudio; `set_effect`
/// só troca o alvo e um fade curto esconde a troca.
pub struct Processor {
 current: Effect,
 target: Effect,
 gain: f64,
 dip: i32,
 dl: Vec<f32>,
 dn: usize,
 n: u64,
 ring_phase: f64,
}

impl Default for Processor {
 fn default() -> Self {
  Self { current: Effect::Pass, target: Effect::Pass, gain: 1.0, dip: 0, dl: vec![0.0; DELAY_LINE], dn: 0, n: 0, ring_phase: 0.0 }
 }
}

impl Processor {
 /// Pede a troca de efeito; a mudança de fato acontece no meio de um fade curto.
 pub fn set_effect(&mut self, e: Effect) {
  if e != self.target {
   self.target = e;
   self.dip = FADE * 2;
  }
 }
 fn reset_state(&mut self) {
  self.dl.iter_mut().for_each(|s| *s = 0.0);
  // Sincroniza a escrita com o relógio atual: leitura (n) e escrita (dn) andam juntas.
  self.dn = (self.n as usize) % self.dl.len();
  self.ring_phase = 0.0;
 }
 fn write(&mut self, x: f32) {
  self.dl[self.dn] = x;
  self.dn = (self.dn + 1) % self.dl.len();
 }
 /// Lê com interpolação linear numa posição fracionária do passado.
 fn read(&self, pos: f64) -> f32 {
  let l = self.dl.len() as f64;
  let mut p = pos % l;
  if p < 0.0 {
   p += l;
  }
  let i = p as usize;
  let f = p - i as f64;
  let a = self.dl[i] as f64;
  let b = self.dl[(i + 1) % self.dl.len()] as f64;
  (a * (1.0 - f) + b * f) as f32
 }
 fn pitch(&mut self, x: f32, ratio: f64) -> f32 {
  let n = self.n as f64;
  self.write(x);
  let p = SWEEP;
  let ph1 = (n % p) / p;
  let ph2 = ((n + p / 2.0) % p) / p;
  // A inclinação do atraso faz o desvio de afinação; base garante ler sempre o passado.
  let slope = 1.0 - ratio;
  let d1 = p + slope * ph1 * p;
  let d2 = p + slope * ph2 * p;
  let g1 = 1.0 - (2.0 * ph1 - 1.0).abs();
  let g2 = 1.0 - (2.0 * ph2 - 1.0).abs();
  let s1 = self.read(n - d1);
  let s2 = self.read(n - d2);
  (s1 as f64 * g1 + s2 as f64 * g2) as f32
 }
 fn ring(&mut self, x: f32, freq: f64, depth: f64) -> f32 {
  let depth = depth.clamp(0.0, 1.0);
  let s = (self.ring_phase * 2.0 * std::f64::consts::PI).sin();
  self.ring_phase += freq / RATE as f64;
  if self.ring_phase >= 1.0 {
   self.ring_phase -= 1.0;
  }
  (x as f64 * ((1.0 - depth) + depth * s)) as f32
 }
 /// Voz engraçada: o ponto de leitura balança para trás e para frente → vibrato cartoon.
 fn wobble(&mut self, x: f32, rate: f64, depth: f64) -> f32 {
  let n = self.n as f64;
  self.write(x);
  let lfo = (self.ring_phase * 2.0 * std::f64::consts::PI).sin();
  self.ring_phase += rate / RATE as f64;
  if self.ring_phase >= 1.0 {
   self.ring_phase -= 1.0;
  }
  self.read(n - (SWEEP + depth * lfo))
 }
 fn echo(&mut self, x: f32, delay_ms: f64, feedback: f64, mix: f64) -> f32 {
  let d = ((delay_ms.max(1.0) / 1000.0) * RATE as f64) as u64;
  let delayed = self.read(self.n as f64 - d as f64);
  self.write(x + delayed * feedback.clamp(0.0, 0.9) as f32);
  (x as f64 * (1.0 - mix) + delayed as f64 * mix) as f32
 }
 /// Processa uma amostra mono. Mantém todo o estado entre amostras.
 pub fn next(&mut self, x: f32) -> f32 {
  if self.dip > 0 {
   if self.dip == FADE {
    self.current = self.target;
    self.reset_state();
   }
   let t = 1.0 - self.dip as f64 / (FADE as f64 * 2.0);
   self.gain = (2.0 * t - 1.0).abs().clamp(0.0, 1.0);
   self.dip -= 1;
  }
  let out = match self.current {
   Effect::Pass => x,
   Effect::Pitch(r) => self.pitch(x, r),
   Effect::Ring(f, d) => self.ring(x, f, d),
   Effect::Wobble(r, d) => self.wobble(x, r, d),
   Effect::Echo(a, f, m) => self.echo(x, a, f, m),
  };
  self.n += 1;
  (out as f64 * self.gain) as f32
 }
}

#[cfg(test)]
mod dsp_tests {
 use super::*;

 fn run(e: Effect, input: &[f32]) -> Vec<f32> {
  let mut p = Processor::default();
  p.set_effect(e);
  // Passa pelo fade inicial até o efeito valer.
  input.iter().map(|&x| p.next(x)).collect()
 }
 /// Estima a frequência contando passagens por zero (sinal limitado a ±1).
 fn zero_freq(out: &[f32], sr: f64) -> f64 {
  let mut crossings = 0;
  for w in out.windows(2) {
   if w[0] <= 0.0 && w[1] > 0.0 {
    crossings += 1;
   }
  }
  crossings as f64 * sr / out.len() as f64
 }
 fn sine(freq: f64, n: usize) -> Vec<f32> {
  (0..n).map(|i| (i as f64 / RATE as f64 * 2.0 * std::f64::consts::PI * freq).sin() as f32).collect()
 }

 #[test]
 fn pass_keeps_the_signal() {
  let input = sine(220.0, 4096);
  let out = run(Effect::Pass, &input);
  // Amostra tardia, depois do atraso natural: deve ser igual à entrada.
  for i in 3000..4096 {
   assert!((out[i] - input[i]).abs() < 1e-3, "Pass alterou a amostra {i}");
  }
 }

 #[test]
 fn pitch_up_raises_the_pitch() {
  let input = sine(200.0, 24_000);
  let out = run(Effect::Pitch(2.0), &input);
  let f = zero_freq(&out[2000..], RATE as f64);
  assert!((f - 400.0).abs() < 120.0, "esperava ~400 Hz, veio {f}");
 }

 #[test]
 fn pitch_down_lowers_the_pitch() {
  let input = sine(400.0, 24_000);
  let out = run(Effect::Pitch(0.5), &input);
  let f = zero_freq(&out[2000..], RATE as f64);
  assert!((f - 200.0).abs() < 120.0, "esperava ~200 Hz, veio {f}");
 }

 #[test]
 fn ring_modulation_produces_the_lfo() {
  // Entrada constante vira a onda do anel (frequência medida por zero crossings).
  let input = vec![1.0f32; 24_000];
  let out = run(Effect::Ring(1000.0, 1.0), &input);
  let f = zero_freq(&out[2000..], RATE as f64);
  assert!((f - 1000.0).abs() < 150.0, "esperava ~1000 Hz, veio {f}");
 }

 #[test]
 fn engracada_e_robo_sao_vozes_diferentes() {
  // Regressão: engracada era mapeada para Ring e voltava como "robo" na UI.
  assert!(matches!(effect_for("robo"), Effect::Ring(..)));
  assert!(matches!(effect_for("engracada"), Effect::Wobble(..)));
  assert_eq!(s_effect_name(&effect_for("robo")), "robo");
  assert_eq!(s_effect_name(&effect_for("engracada")), "engracada");
 }

 #[test]
 fn wobble_passa_sinal_e_fica_limitado() {
  let input = sine(200.0, 24_000);
  let out = run(Effect::Wobble(9.0, SWEEP * 0.16), &input);
  let tail = &out[6000..];
  let peak = tail.iter().fold(0.0f32, |m, &s| m.max(s.abs()));
  assert!(peak > 0.3, "engraçada deveria passar sinal, pico {peak}");
  assert!(peak < 1.6, "engraçada não deve explodir, pico {peak}");
  // Vibrato oscila ao redor da original: a média continua perto de 200 Hz.
  let f = zero_freq(tail, RATE as f64);
  assert!((f - 200.0).abs() < 100.0, "esperava ~200 Hz na média, veio {f}");
 }

 #[test]
 fn echo_repeats_the_impulse() {
  let mut input = vec![0.0f32; 12_000];
  // Depois do fade inicial, com o eco já ativo.
  input[1000] = 1.0;
  let out = run(Effect::Echo(50.0, 0.0, 1.0), &input);
  // 50 ms em 48 kHz = 2400 amostras depois do impulso (1000 + 2400 = 3400).
  let peak = out[3300..3500].iter().cloned().fold(0.0f32, f32::max);
  assert!(peak > 0.5, "eco não apareceu perto da amostra 3400 (pico {peak})");
 }

 #[test]
 fn switching_effect_does_not_explode() {
  let input = sine(220.0, 8000);
  let mut p = Processor::default();
  p.set_effect(Effect::Pitch(1.5));
  let out: Vec<f32> = input.iter().map(|&x| {
   p.next(x)
  }).collect();
  assert!(out.iter().all(|v| v.is_finite() && v.abs() <= 4.0), "saiu fora dos limites");
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn downmix_stereo_averages_channels() {
  let stereo = [0.4f32, 0.6, 0.2, 0.8];
  let mono = downmix(&stereo, 2);
  assert_eq!(mono, vec![0.5, 0.5]);
 }

 #[test]
 fn downmix_tolerates_zero_channels() {
  // Canal 0 é tratado como mono: cada amostra passa direto, sem média.
  assert_eq!(downmix(&[0.3f32, 0.9], 0), vec![0.3, 0.9]);
 }

 #[test]
 fn upmix_duplicates_mono_into_every_channel() {
  let mono = [0.5f32, -0.25];
  let mut out = vec![0f32; 4];
  upmix(&mono, 2, &mut out);
  assert_eq!(out, vec![0.5, 0.5, -0.25, -0.25]);
 }

 #[test]
 fn upmix_never_writes_past_the_buffer() {
  let mono = [1f32; 8];
  let mut out = vec![0f32; 3];
  upmix(&mono, 2, &mut out);
  assert_eq!(out.len(), 3);
 }

 #[test]
 fn peak_takes_largest_absolute_sample() {
  assert_eq!(peak(&[0.1f32, -0.9, 0.5]), 0.9);
  assert_eq!(peak(&[]), 0.0);
 }

 #[test]
 fn default_status_is_off() {
  let s = Status::default();
  assert_eq!(s.phase, OFF);
  assert!(!s.input.contains("controlapi"), "nunca vaza credencial");
 }
}

#[cfg(test)]
mod resample_tests {
 use super::*;
 fn close(a: f64, b: f64) -> bool { (a - b).abs() / b.max(1.0) < 0.02 }
 #[test]
 fn taxa_igual_e_passthrough() {
  let mut r = Resample::new(48000, 48000);
  let input: Vec<f32> = (0..100).map(|i| (i as f32 * 0.3).sin()).collect();
  let mut out = Vec::new();
  r.convert(&input, &mut out);
  assert!(out.len() >= 98, "passthrough deve manter a contagem: {}", out.len());
  for (i, &s) in out.iter().enumerate() { assert!((s - input[i]).abs() < 1e-5, "amostra {i} distorcida"); }
 }
 #[test]
 fn upsample_amplia_e_downsample_reduz() {
  let mut up = Resample::new(44100, 48000);
  let mut out = Vec::new();
  up.convert(&vec![0.5f32; 4410], &mut out);
  assert!(close(out.len() as f64, 4410.0 * 48000.0 / 44100.0), "44,1→48 deve ampliar, veio {}", out.len());
  assert!(out.len() > 4410);
  let mut down = Resample::new(48000, 16000);
  let mut out = Vec::new();
  down.convert(&vec![0.5f32; 4800], &mut out);
  assert!(close(out.len() as f64, 4800.0 * 16000.0 / 48000.0), "48→16 deve reduzir, veio {}", out.len());
  assert!(out.len() < 4800);
 }
 #[test]
 fn pull_sempre_devolve_o_que_pedi() {
  let mut r = Resample::new(48000, 48000);
  let mut out = Vec::new();
  r.pull(256, &mut out);
  assert_eq!(out.len(), 256, "mesmo sem dado, devolve a contagem pedida");
 }
}

#[cfg(test)]
mod gate_tests {
 use super::*;

 /// Fundo baixo e determinístico (sem rand: o teste é sempre igual).
 fn noise(n: usize, amp: f32) -> Vec<f32> {
  (0..n).map(|i| amp * ((i * 7919 % 997) as f32 / 997.0 - 0.5)).collect()
 }
 fn tone(freq: f64, n: usize) -> Vec<f32> {
  (0..n).map(|i| (i as f64 / RATE as f64 * 2.0 * std::f64::consts::PI * freq).sin() as f32).collect()
 }
 fn run_gate(thr: f32, input: &[f32]) -> Vec<f32> {
  let mut g = Gate::new(thr);
  input.iter().map(|&x| g.process(x)).collect()
 }

 #[test]
 fn gate_corta_o_fundo_quando_silencio() {
  let out = run_gate(db_to_amp(-45.0), &noise(48_000, 0.002));
  let tail: f32 = out[40_000..].iter().map(|s| s.abs()).sum();
  assert!(tail < 1e-3, "fundo deveria ficar mudo, soma {tail}");
 }

 #[test]
 fn gate_abre_com_voz_e_passa_integra() {
  let mut input = vec![0.0f32; 2000];
  input.extend(tone(220.0, 20_000));
  let out = run_gate(db_to_amp(-45.0), &input);
  // Depois do ataque (~5 ms) a voz deve passar intacta.
  for i in 2500..21_999 {
   assert!((out[i] - input[i]).abs() < 0.02, "voz distorcida na amostra {i}");
  }
 }

 #[test]
 fn gate_segura_o_fim_da_palavra() {
  let mut input = vec![0.0f32; 4800];
  input.extend(vec![1.0f32; 2400]);
  input.extend(tone(220.0, 2400).iter().map(|s| s * 0.05));
  input.extend(noise(20_000, 0.001));
  let out = run_gate(db_to_amp(-45.0), &input);
  // A sílaba baixa logo depois da fala alta passa inteira (hold)…
  for i in 7800..9500 {
   assert!((out[i] - input[i]).abs() < 0.02, "cortou a sílaba baixa na amostra {i}");
  }
  // …e depois de ~1 s só de fundo, a porta fechou.
  let tail: f32 = out[29_000..].iter().map(|s| s.abs()).sum();
  assert!(tail < 1e-3, "deveria ter fechado, soma {tail}");
 }

 #[test]
 fn gate_nunca_amplifica() {
  let mut input: Vec<f32> = tone(150.0, 4000).iter().map(|s| s * 0.8).collect();
  input.extend(noise(4000, 0.3));
  let out = run_gate(db_to_amp(-45.0), &input);
  for (i, &s) in out.iter().enumerate() {
   assert!(s.abs() <= input[i].abs() + 1e-6, "gate amplificou na amostra {i}");
  }
 }

 #[test]
 fn gate_ajustavel_corta_mais_com_limiar_alto() {
  // -20 dBFS ≈ 0.1: sinal de 0.05 fica mudo; com limiar baixo, passa.
  let input = vec![0.05f32; 24_000];
  let out = run_gate(db_to_amp(-20.0), &input);
  let tail: f32 = out[20_000..].iter().map(|s| s.abs()).sum();
  assert!(tail < 1e-3, "limiar alto deveria cortar, soma {tail}");
  let out = run_gate(db_to_amp(-34.0), &input);
  assert!((out[23_000] - 0.05).abs() < 1e-4, "limiar baixo deveria deixar passar");
 }

 #[test]
 fn db_para_amplitude_e_o_esperado() {
  assert!((db_to_amp(0.0) - 1.0).abs() < 1e-6);
  assert!((db_to_amp(-20.0) - 0.1).abs() < 1e-6);
  assert!((db_to_amp(-45.0) - 0.00562).abs() < 1e-4);
 }
}
