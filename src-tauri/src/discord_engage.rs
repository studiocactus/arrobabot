use crate::engine::Runtime;
use crate::model::*;
use crate::{discord, discord_admin};
use rand::Rng;
use serde_json::{json, Value};
use chrono::Datelike;
use std::sync::Arc;

fn xp_key(p: &str) -> String {
	format!("discord_xp:{p}")
}
fn bday_key(p: &str) -> String {
	format!("discord_birthdays:{p}")
}
fn give_key(p: &str) -> String {
	format!("discord_giveaways:{p}")
}
fn link_key(p: &str) -> String {
	format!("discord_links:{p}")
}
fn read(rt: &Runtime, key: &str) -> Value {
	rt.db.get(key)
}
fn write(rt: &Runtime, key: &str, v: &Value) -> Result<(), String> {
	rt.db.set(key, v)
}

// ---------------------------------------------------------------- XP

/// Level curve: 100 XP for level 1, 400 for level 2, 900 for level 3.
pub fn level(xp: u64) -> u64 {
	(xp as f64 / 100.0).sqrt() as u64
}
pub fn next_level_xp(l: u64) -> u64 {
	(l + 1) * (l + 1) * 100
}
/// Grants XP for a Discord message, at most once a minute per member.
pub fn on_chat(rt: &Runtime, p: &Profile, cfg: &Value, uid: &str, user: &str, _content: &str) {
	if cfg["xp"]["enabled"] != true || uid.is_empty() {
		return;
	}
	let key = xp_key(&p.id);
	let mut store = read(rt, &key);
	if !store.is_object() {
		store = json!({"users":{}});
	}
	let now = chrono::Utc::now().timestamp();
	let current = store["users"][uid]["xp"].as_u64().unwrap_or(0);
	let last = store["users"][uid]["last"].as_i64().unwrap_or(0);
	if now - last < 60 {
		return;
	}
	let gain = cfg["xp"]["perMessage"].as_u64().unwrap_or(15).clamp(1, 500);
	let users = store["users"].as_object_mut().expect("xp store is an object");
	if users.len() >= 500 && !users.contains_key(uid) {
		return;
	}
	users.insert(uid.to_owned(), json!({"xp": current + gain, "name": user, "last": now}));
	if write(rt, &key, &store).is_ok() {
		rt.emit("discord", json!({"profileId": p.id, "kind": "xp"}));
	}
}
fn rank(rt: &Runtime, p: &str, limit: usize) -> Vec<Value> {
	let store = read(rt, &xp_key(p));
	let mut rows: Vec<Value> = store["users"]
		.as_object()
		.map(|m| {
			m.iter()
				.map(|(id, v)| {
					let xp = v["xp"].as_u64().unwrap_or(0);
					let lvl = level(xp);
					json!({"id": id, "name": v["name"], "xp": xp, "level": lvl, "next": next_level_xp(lvl)})
				})
				.collect()
		})
		.unwrap_or_default();
	rows.sort_by(|a, b| b["xp"].as_u64().unwrap_or(0).cmp(&a["xp"].as_u64().unwrap_or(0)));
	rows.truncate(limit.max(1));
	rows
}

// ---------------------------------------------------------------- Birthdays

/// Accepts `DD/MM` or `DD/MM/YYYY`.
pub fn parse_date(raw: &str) -> Option<(u32, u32, Option<i32>)> {
	let parts: Vec<&str> = raw.trim().split('/').collect();
	if parts.len() != 2 && parts.len() != 3 {
		return None;
	}
	let day = parts[0].trim().parse().ok()?;
	let month = parts[1].trim().parse().ok()?;
	if day == 0 || day > 31 || month == 0 || month > 12 {
		return None;
	}
	let year = if parts.len() == 3 { parts[2].trim().parse().ok() } else { None };
	Some((day, month, year))
}
pub fn is_today(day: u32, month: u32, now: &chrono::DateTime<chrono::Utc>) -> bool {
	now.day() == day && now.month() == month
}

// ---------------------------------------------------------------- Giveaways

fn pick_winners(entries: &[Value], count: usize, exclude: &[Value]) -> Vec<Value> {
	let mut pool: Vec<Value> = entries.iter().filter(|e| !exclude.contains(e)).cloned().collect();
	let mut out = vec![];
	let mut rng = rand::thread_rng();
	while out.len() < count && !pool.is_empty() {
		let i = rng.gen_range(0..pool.len());
		out.push(pool.remove(i));
	}
	out
}
fn mention_all(ids: &[Value]) -> String {
	ids.iter()
		.filter_map(|w| w.as_str().map(|id| format!("<@{id}>")))
		.collect::<Vec<_>>()
		.join(", ")
}

/// Adds a member to the giveaway that the reacted message belongs to.
pub fn reaction(rt: &Runtime, p: &Profile, cfg: &Value, d: &Value) {
	if cfg["giveaway"]["enabled"] != true || d["emoji"]["name"].as_str() != Some("🎉") {
		return;
	}
	let uid = d["user_id"].as_str().unwrap_or("");
	if uid.is_empty() || d["member"]["user"]["bot"] == true {
		return;
	}
	let key = give_key(&p.id);
	let mut list = read(rt, &key).as_array().cloned().unwrap_or_default();
	let mut changed = false;
	for g in list.iter_mut() {
		if g["status"].as_str() != Some("open") {
			continue;
		}
		if g["messageId"].as_str() != d["message_id"].as_str()
			|| g["channelId"].as_str() != d["channel_id"].as_str()
		{
			continue;
		}
		let entries = g["entries"].as_array_mut().unwrap();
		if !entries.iter().any(|e| e.as_str() == Some(uid)) {
			entries.push(json!(uid));
			changed = true;
		}
	}
	if changed {
		let _ = write(rt, &key, &Value::Array(list));
		rt.emit("discord", json!({"profileId": p.id, "kind": "giveaway"}));
	}
}

// ---------------------------------------------------------------- chat bridge

/// `!vincular CODIGO` typed on Twitch links the viewer to the Discord member that issued the code.
pub fn link_command(rt: &Runtime, p: &Profile, e: &Event) -> Option<String> {
	if e.kind != "chat" || e.user_id.is_empty() {
		return None;
	}
	// The pairing code is typed on Twitch; Discord already knows who is speaking.
	if e.data["source"].as_str() == Some("discord") {
		return None;
	}
	let mut parts = e.message.split_whitespace();
	if parts.next()?.to_lowercase() != "!vincular" {
		return None;
	}
	let code = parts.next()?.to_uppercase();
	let key = link_key(&p.id);
	let mut store = read(rt, &key);
	if !store.is_object() {
		store = json!({"pending":{},"links":[]});
	}
	let entry = store["pending"].get(&code)?.clone();
	store["pending"].as_object_mut()?.remove(&code);
	let mut links = store["links"].as_array().cloned().unwrap_or_default();
	links.push(json!({"twitchId": e.user_id, "discordId": entry["discordId"], "at": chrono::Utc::now().to_rfc3339()}));
	store["links"] = Value::Array(links);
	let _ = write(rt, &key, &store);
	rt.emit("discord", json!({"profileId": p.id, "kind": "link"}));
	Some(format!("Conta vinculada com sucesso, {}.", e.user))
}
fn identities(rt: &Runtime, p: &str) -> Vec<Value> {
	read(rt, &link_key(p))["links"].as_array().cloned().unwrap_or_default()
}

// ---------------------------------------------------------------- scheduler

pub async fn tick(rt: &Arc<Runtime>, p: &Profile) {
	let cfg = discord::config(rt, &p.id);
	if cfg["enabled"] != true {
		return;
	}
	let now = chrono::Utc::now();
	// Birthdays
	if cfg["birthday"]["enabled"] == true
		&& cfg["birthday"]["channelId"].as_str().is_some_and(|c| !c.is_empty())
	{
		let key = bday_key(&p.id);
		let mut list = read(rt, &key).as_array().cloned().unwrap_or_default();
		let mut changed = false;
		for b in list.iter_mut() {
			let day = b["day"].as_u64().unwrap_or(0) as u32;
			let month = b["month"].as_u64().unwrap_or(0) as u32;
			if day == 0 || !is_today(day, month, &now) {
				continue;
			}
			if b["lastYear"].as_i64() == Some(now.year() as i64) {
				continue;
			}
			let name = b["name"].as_str().unwrap_or("").to_owned();
			let uid = b["user"].as_str().unwrap_or("");
			let age = match b["year"].as_i64() {
				Some(y) => (now.year() as i64 - y).max(0),
				None => 0,
			};
			let template = cfg["birthday"]["text"].as_str().unwrap_or("");
			let text = if template.trim().is_empty() {
				format!("🎂 Feliz aniversário, {name}!")
			} else {
				template
					.replace("{user}", &format!("<@{uid}>"))
					.replace("{name}", &name)
					.replace("{age}", &age.to_string())
			};
			let channel = cfg["birthday"]["channelId"].as_str().unwrap_or("");
			match discord::post(rt, p, channel, &text).await {
				Ok(_) => {
					b["lastYear"] = json!(now.year());
					changed = true;
					rt.log(&p.id, "discord", &format!("Aniversário de {name} comemorado"), "success");
				}
				Err(err) => rt.log(&p.id, "discord", &err, "error"),
			}
		}
		if changed {
			let _ = write(rt, &key, &Value::Array(list));
		}
	}
	// Giveaways
	if cfg["giveaway"]["enabled"] == true {
		let key = give_key(&p.id);
		let mut list = read(rt, &key).as_array().cloned().unwrap_or_default();
		let mut changed = false;
		for g in list.iter_mut() {
			if g["status"].as_str() != Some("open") || g["endsAt"].as_i64().unwrap_or(0) > now.timestamp() {
				continue;
			}
			g["status"] = json!("ended");
			changed = true;
			let count = g["winners"].as_u64().unwrap_or(1).clamp(1, 20) as usize;
			let entries = g["entries"].as_array().cloned().unwrap_or_default();
			let winners = pick_winners(&entries, count, &[]);
			let prize = g["prize"].as_str().unwrap_or("prêmio").to_owned();
			let channel = g["channelId"].as_str().unwrap_or("");
			let mentioned = mention_all(&winners);
			let text = if mentioned.is_empty() {
				format!("🎲 O sorteio de **{prize}** terminou e ninguém participou.")
			} else {
				format!("🎲 O sorteio de **{prize}** terminou! Vencedor(es): {mentioned}")
			};
			let _ = discord::post(rt, p, channel, &text).await;
			g["winnersList"] = Value::Array(winners);
			rt.log(&p.id, "discord", &format!("Sorteio de {prize} encerrado"), "success");
		}
		if changed {
			let _ = write(rt, &key, &Value::Array(list));
			rt.emit("discord", json!({"profileId": p.id, "kind": "giveaway"}));
		}
	}
}

// ---------------------------------------------------------------- engagement actions

pub async fn action(
	rt: &Arc<Runtime>,
	p: &str,
	profile: &Profile,
	args: &Value,
) -> Result<Value, String> {
	let cfg = discord::config(rt, p);
	let act = args["action"].as_str().ok_or("Ação ausente")?;
	let target = args["target"].as_str().unwrap_or("");
	match act {
		// ---- XP
		"xpRank" => Ok(Value::Array(rank(rt, p, args["limit"].as_u64().unwrap_or(20).clamp(1, 100) as usize))),
		"xpAdd" | "xpEdit" | "xpTransfer" => change_xp(rt, p, act, args),
		// ---- birthdays
		"birthdayAdd" | "birthdayRemove" | "birthdayList" | "birthdayToday" => {
			let key = bday_key(p);
			let mut list = read(rt, &key).as_array().cloned().unwrap_or_default();
			match act {
				"birthdayList" => return Ok(Value::Array(list)),
				"birthdayToday" => {
					let now = chrono::Utc::now();
					let today: Vec<Value> = list
						.into_iter()
						.filter(|b| is_today(b["day"].as_u64().unwrap_or(0) as u32, b["month"].as_u64().unwrap_or(0) as u32, &now))
						.collect();
					return Ok(Value::Array(today));
				}
				"birthdayRemove" => {
					let before = list.len();
					list.retain(|b| b["user"].as_str() != Some(target));
					if list.len() == before {
						return Err("Aniversário não encontrado".into());
					}
				}
				_ => {
					if target.is_empty() {
						return Err("Escolha o membro".into());
					}
					let raw = args["date"].as_str().ok_or("Informe a data como DD/MM ou DD/MM/AAAA")?;
					let (day, month, year) =
						parse_date(raw).ok_or("Data inválida. Use DD/MM ou DD/MM/AAAA.")?;
					list.retain(|b| b["user"].as_str() != Some(target));
					if list.len() >= 500 {
						return Err("Limite de 500 aniversariantes por perfil".into());
					}
					list.push(json!({"user":target,"name":args["targetName"].as_str().unwrap_or(target),"day":day,"month":month,"year":year,"lastYear":0}));
				}
			}
			write(rt, &key, &Value::Array(list))?;
			rt.emit("discord", json!({"profileId": p, "kind": "birthday"}));
			Ok(Value::Null)
		}
		// ---- giveaways
		"giveawayList" => Ok(read(rt, &give_key(p))),
		"giveawayStart" | "giveawayEdit" | "giveawayEnd" | "giveawayReroll" => {
			giveaway(rt, p, profile, &cfg, act, args).await
		}
		// ---- identity link (A)
		"linkCreate" => {
			if target.is_empty() {
				return Err("Escolha o membro".into());
			}
			let key = link_key(p);
			let mut store = read(rt, &key);
			if !store.is_object() {
				store = json!({"pending":{},"links":[]});
			}
			let code = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000));
			let entry = json!({"discordId": target, "at": chrono::Utc::now().to_rfc3339()});
			store["pending"]
				.as_object_mut()
				.ok_or("Vínculos indisponíveis")?
				.insert(code.clone(), entry);
			write(rt, &key, &store)?;
			rt.emit("discord", json!({"profileId": p, "kind": "link"}));
			Ok(json!({"code": code, "hint": format!("Peça ao espectador digitar !vincular {code} no chat da Twitch")}))
		}
		"linkList" => Ok(json!({"links": identities(rt, p), "pending": read(rt, &link_key(p))["pending"]})),
		"linkRemove" => {
			let key = link_key(p);
			let mut store = read(rt, &key);
			let links = store["links"].as_array_mut().ok_or("Vínculos indisponíveis")?;
			let before = links.len();
			links.retain(|l| l["discordId"].as_str() != Some(target) && l["twitchId"].as_str() != Some(target));
			if links.len() == before {
				return Err("Vínculo não encontrado".into());
			}
			write(rt, &key, &store)?;
			Ok(Value::Null)
		}
		"linkIdentity" => {
			let found: Vec<Value> = identities(rt, p)
				.into_iter()
				.filter(|l| l["discordId"].as_str() == Some(target) || l["twitchId"].as_str() == Some(target))
				.collect();
			let mut discord_ids = vec![];
			let mut twitch_ids = vec![];
			for l in &found {
				if let Some(d) = l["discordId"].as_str() {
					discord_ids.push(d.to_owned());
				}
				if let Some(t) = l["twitchId"].as_str() {
					twitch_ids.push(t.to_owned());
				}
			}
			Ok(json!({"links": found, "discordIds": discord_ids, "twitchIds": twitch_ids, "records": found.len()}))
		}
		_ => Err("Ação de engajamento desconhecida".into()),
	}
}

fn change_xp(rt: &Runtime, p: &str, act: &str, args: &Value) -> Result<Value, String> {
	let key = xp_key(p);
	let mut store = read(rt, &key);
	if !store.is_object() {
		store = json!({"users":{}});
	}
	let users = store["users"].as_object_mut().ok_or("XP indisponível")?;
	let target = args["target"].as_str().unwrap_or("");
	if act == "xpTransfer" {
		let to = args["to"].as_str().ok_or("Escolha o destino")?;
		let from = users.get(target).cloned().ok_or("Membro sem XP registrado")?;
		let amount = from["xp"].as_u64().unwrap_or(0);
		let dest = users.get(to).cloned().unwrap_or(json!({"xp":0,"name":to,"last":0}));
		let total = dest["xp"].as_u64().unwrap_or(0) + amount;
		users.insert(
			to.to_owned(),
			json!({"xp": total, "name": dest["name"], "last": dest["last"]}),
		);
		users.remove(target);
		write(rt, &key, &store)?;
		rt.emit("discord", json!({"profileId": p, "kind": "xp"}));
		return Ok(json!({"moved": amount, "xp": total, "level": level(total)}));
	}
	if target.is_empty() {
		return Err("Escolha o membro".into());
	}
	let value = args["value"].as_i64().ok_or("Informe um valor")?;
	let current = users
		.get(target)
		.cloned()
		.unwrap_or(json!({"xp":0,"name":args["targetName"].as_str().unwrap_or(target),"last":0}));
	let base = current["xp"].as_u64().unwrap_or(0) as i64;
	let next = if act == "xpEdit" { value } else { base + value }.max(0) as u64;
	users.insert(target.to_owned(), json!({"xp": next, "name": current["name"], "last": current["last"]}));
	write(rt, &key, &store)?;
	rt.emit("discord", json!({"profileId": p, "kind": "xp"}));
	Ok(json!({"xp": next, "level": level(next)}))
}

async fn giveaway(
	rt: &Arc<Runtime>,
	p: &str,
	profile: &Profile,
	cfg: &Value,
	act: &str,
	args: &Value,
) -> Result<Value, String> {
	let key = give_key(p);
	let mut list = read(rt, &key).as_array().cloned().unwrap_or_default();
	match act {
		"giveawayStart" => {
			let channel = args["channel"].as_str().ok_or("Escolha um canal")?;
			let prize = args["prize"].as_str().unwrap_or("").trim();
			if prize.is_empty() {
				return Err("Dê um nome ao prêmio".into());
			}
			let minutes = args["minutes"]
				.as_u64()
				.unwrap_or(cfg["giveaway"]["minutes"].as_u64().unwrap_or(10))
				.clamp(1, 10080);
			let winners = args["winners"].as_u64().unwrap_or(1).clamp(1, 20);
			let text = format!(
				"🎉 **Sorteio!** {prize}\nReaja com 🎉 para participar · {minutes} minuto(s) · {winners} vencedor(es)"
			);
			let sent = discord::post(rt, profile, channel, &text).await?;
			list.insert(
				0,
				json!({"id":uuid::Uuid::new_v4().to_string(),"channelId":channel,
					"messageId":sent["id"].as_str().unwrap_or(""),"prize":prize,
					"endsAt":chrono::Utc::now().timestamp()+minutes as i64*60,
					"winners":winners,"status":"open","entries":[]}),
			);
		}
		"giveawayEnd" | "giveawayReroll" => {
			let id = args["id"].as_str().ok_or("Selecione o sorteio")?;
			let g = list.iter_mut().find(|g| g["id"].as_str() == Some(id)).ok_or("Sorteio não encontrado")?;
			let count = g["winners"].as_u64().unwrap_or(1).clamp(1, 20) as usize;
			let entries = g["entries"].as_array().cloned().unwrap_or_default();
			if entries.is_empty() {
				return Err("Ninguém participou deste sorteio".into());
			}
			let already = g["winnersList"].as_array().cloned().unwrap_or_default();
			let winners = pick_winners(&entries, count, &already);
			if winners.is_empty() {
				return Err("Todos já foram sorteados. Reinicie o sorteio.".into());
			}
			let channel = g["channelId"].as_str().unwrap_or("").to_owned();
			let prize = g["prize"].as_str().unwrap_or("").to_owned();
			g["status"] = json!("ended");
			g["endsAt"] = json!(chrono::Utc::now().timestamp());
			let mentioned = mention_all(&winners);
			let _ = discord::post(
				rt,
				profile,
				&channel,
				&format!("🎲 Novo vencedor do sorteio de **{prize}**: {mentioned}"),
			)
			.await;
			g["winnersList"] = Value::Array(winners);
		}
		_ => {
			let id = args["id"].as_str().ok_or("Selecione o sorteio")?;
			let g = list.iter_mut().find(|g| g["id"].as_str() == Some(id)).ok_or("Sorteio não encontrado")?;
			if let Some(prize) = args["prize"].as_str() {
				if !prize.trim().is_empty() {
					g["prize"] = json!(prize);
				}
			}
			if let Some(m) = args["minutes"].as_u64() {
				g["endsAt"] = json!(chrono::Utc::now().timestamp() + m.clamp(1, 10080) as i64 * 60);
				g["status"] = json!("open");
			}
		}
	}
	write(rt, &key, &Value::Array(list))?;
	rt.emit("discord", json!({"profileId": p, "kind": "giveaway"}));
	Ok(read(rt, &key))
}

// ---------------------------------------------------------------- slash commands

const P_ADMIN: u64 = 1 << 3;
const P_KICK: u64 = 1 << 1;
const P_BAN: u64 = 1 << 2;
const P_MODERATE: u64 = 1 << 40;

fn perms(m: &Value) -> u64 {
	m["permissions"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0)
}
fn can_moderate(m: &Value) -> bool {
	perms(m) & (P_ADMIN | P_KICK | P_BAN | P_MODERATE) != 0
}
fn opt<'a>(d: &'a Value, name: &str) -> Option<&'a Value> {
	d["options"].as_array()?.iter().find(|o| o["name"].as_str() == Some(name))
}
fn opt_str(d: &Value, name: &str) -> String {
	opt(d, name).and_then(|o| o["value"].as_str()).unwrap_or("").to_owned()
}
fn opt_int(d: &Value, name: &str) -> Option<u64> {
	opt(d, name).and_then(|o| o["value"].as_u64())
}
fn opt_name(d: &Value, name: &str) -> String {
	let id = opt_str(d, name);
	if id.is_empty() {
		return String::new();
	}
	d["resolved"]["users"][id.as_str()]["username"]
		.as_str()
		.map(str::to_owned)
		.unwrap_or(id)
}
fn command_list() -> Value {
	json!([
		{"name":"painel","description":"Mostra o estado do BotLive neste servidor","type":1},
		{"name":"convite","description":"Gera o link para adicionar o BotLive em outro servidor","type":1},
		{"name":"ranking","description":"Ranking de XP do servidor","type":1},
		{"name":"limpar","description":"Apaga ate 100 mensagens deste canal","type":1,
		 "options":[{"name":"quantidade","description":"De 1 a 100","type":4,"required":false}]},
		{"name":"ban","description":"Bane um membro do servidor","type":1,
		 "options":[{"name":"usuario","description":"Membro a banir","type":6,"required":true},
		           {"name":"motivo","description":"Motivo registrado na auditoria","type":3,"required":false}]},
		{"name":"mute","description":"Silencia um membro temporariamente","type":1,
		 "options":[{"name":"usuario","description":"Membro a silenciar","type":6,"required":true},
		           {"name":"minutos","description":"Duracao em minutos","type":4,"required":false},
		           {"name":"motivo","description":"Motivo registrado na auditoria","type":3,"required":false}]},
		{"name":"warn","description":"Registra um aviso em um membro","type":1,
		 "options":[{"name":"usuario","description":"Membro avizado","type":6,"required":true},
		           {"name":"motivo","description":"Motivo do aviso","type":3,"required":false}]},
		{"name":"avisos","description":"Lista os avisos de um membro","type":1,
		 "options":[{"name":"usuario","description":"Membro consultado","type":6,"required":true}]},
		{"name":"xp","description":"Mostra o nivel de um membro","type":1,
		 "options":[{"name":"usuario","description":"Membro consultado","type":6,"required":false}]},
		{"name":"aniversario","description":"Registra seu aniversario no servidor","type":1,
		 "options":[{"name":"data","description":"DD/MM ou DD/MM/AAAA","type":3,"required":true}]},
		{"name":"vincular","description":"Vincula sua conta da Twitch a do Discord","type":1,
		 "options":[{"name":"codigo","description":"Codigo de seis digitos","type":3,"required":true}]},
		{"name":"sorteio","description":"Inicia um sorteio neste canal","type":1,
		 "options":[{"name":"premio","description":"O que sera sorteado","type":3,"required":true},
		           {"name":"minutos","description":"Duracao em minutos","type":4,"required":false},
		           {"name":"vencedores","description":"Quantidade de vencedores","type":4,"required":false}]}
	])
}
/// Publishes the slash commands of the profile inside the configured guild.
pub async fn register(rt: &Arc<Runtime>, p: &Profile, guild: &str, args: &Value) -> Result<Value, String> {
	let app = match args["app"]
		.as_str()
		.map(str::to_owned)
		.filter(|s| !s.is_empty())
	{
		Some(v) => v,
		None => discord::identity(rt, p)
			.await
			.ok()
			.and_then(|v| v["id"].as_str().map(str::to_owned))
			.ok_or_else(|| "Não foi possível identificar o bot".to_string())?,
	};
	if guild.is_empty() {
		return Err("Informe o ID do servidor do Discord".into());
	}
	let commands = match args["commands"].as_array() {
		Some(list) if !list.is_empty() => Value::Array(list.clone()),
		_ => command_list(),
	};
	let out = discord::rest(rt, p, "PUT", &format!("/applications/{app}/guilds/{guild}/commands"), Some(commands)).await?;
	Ok(json!({"registered": out.as_array().map(|a| a.len()).unwrap_or(0)}))
}

/// Answers a slash command that arrived through the gateway.
pub async fn interaction(rt: &Arc<Runtime>, p: &Profile, d: &Value) {
	let app = d["application_id"].as_str().unwrap_or("");
	let token = d["token"].as_str().unwrap_or("");
	let iid = d["id"].as_str().unwrap_or("");
	let name = d["data"]["name"].as_str().unwrap_or("");
	if app.is_empty() || token.is_empty() || iid.is_empty() || name.is_empty() {
		return;
	}
	let cfg = discord::config(rt, &p.id);
	if cfg["enabled"] != true {
		return;
	}
	// Read-only answers reply immediately; anything that hits the Discord API is deferred first.
	let instant = matches!(name, "painel" | "convite" | "xp" | "ranking" | "aniversario" | "vincular" | "avisos");
	let body = json!({"type": if instant {4} else {5}, "data": {"flags": 64}});
	if let Err(err) =
		discord::rest(rt, p, "POST", &format!("/interactions/{iid}/{token}/callback"), Some(body)).await
	{
		rt.log(&p.id, "discord", &err, "error");
		return;
	}
	let answer = run_command(rt, p, &cfg, name, d).await;
	let edit = json!({"content": answer, "flags": 64, "allowed_mentions": {"parse": []}});
	if let Err(err) = discord::rest(
		rt,
		p,
		"PATCH",
		&format!("/webhooks/{app}/{token}/messages/@original"),
		Some(edit),
	)
	.await
	{
		rt.log(&p.id, "discord", &err, "error");
	}
}

async fn run_command(rt: &Arc<Runtime>, p: &Profile, cfg: &Value, name: &str, d: &Value) -> String {
	let guild = cfg["guildId"].as_str().unwrap_or("");
	if guild.is_empty() && name != "painel" && name != "convite" {
		return "Configure o servidor do Discord na tela do Discord antes de usar comandos.".into();
	}
	if matches!(name, "limpar" | "ban" | "mute" | "warn" | "sorteio") && !can_moderate(&d["member"]) {
		return "Você não tem permissão de moderação neste servidor.".into();
	}
	let uid = d["member"]["user"]["id"].as_str().unwrap_or("");
	let target = opt_str(d, "usuario");
	let target_name = opt_name(d, "usuario");
	let reason = opt_str(d, "motivo");
	let base = json!({
		"target": target, "targetName": target_name, "reason": reason,
		"channel": d["channel_id"], "channelId": d["channel_id"], "guild": guild
	});
	let call = |mut a: Value| {
		let mut v = base.clone();
		v.as_object_mut().unwrap().extend(a.as_object_mut().unwrap().clone());
		v
	};
	match name {
		"painel" => format!(
			"**BotLive** neste servidor\nServidor: {}\nSorteios: {}\nXP: {}\nAniversários: {}",
			if guild.is_empty() { "não definido" } else { "definido" },
			if cfg["giveaway"]["enabled"] == true { "ativado" } else { "desativado" },
			if cfg["xp"]["enabled"] == true { "ativado" } else { "desativado" },
			if cfg["birthday"]["enabled"] == true { "ativado" } else { "desativado" },
		),
		"convite" => match discord_admin::action(rt, &p.id, &json!({"action":"invite"})).await {
			Ok(v) => v["url"].as_str().unwrap_or("Link indisponível.").to_owned(),
			Err(err) => err,
		},
		"limpar" => {
			let mut a = call(json!({"action":"clear"}));
			a["limit"] = json!(opt_int(d, "quantidade").unwrap_or(50).clamp(1, 100));
			match discord_admin::action(rt, &p.id, &a).await {
				Ok(v) => format!("{} mensagem(s) apagada(s).", v["removed"].as_u64().unwrap_or(0)),
				Err(err) => err,
			}
		}
		"ban" => match discord_admin::action(rt, &p.id, &call(json!({"action":"ban"}))).await {
			Ok(_) => format!("Membro {target_name} banido."),
			Err(err) => err,
		},
		"mute" => {
			let mut a = call(json!({"action":"timeout"}));
			a["seconds"] = json!(opt_int(d, "minutos").unwrap_or(10).min(40320) * 60);
			match discord_admin::action(rt, &p.id, &a).await {
				Ok(_) => format!("Membro {target_name} silenciado."),
				Err(err) => err,
			}
		}
		"warn" => match discord_admin::action(rt, &p.id, &call(json!({"action":"warn"}))).await {
			Ok(_) => format!("Aviso registrado para {target_name}."),
			Err(err) => err,
		},
		"avisos" => match discord_admin::action(rt, &p.id, &call(json!({"action":"warnlist"}))).await {
			Ok(v) => {
				let list = v.as_array().cloned().unwrap_or_default();
				if list.is_empty() {
					format!("{target_name} não tem avisos.")
				} else {
					let rows: Vec<String> = list
						.iter()
						.take(10)
						.map(|w| format!("• {}", w["reason"].as_str().unwrap_or("sem motivo")))
						.collect();
					format!("Avisos de {target_name}:\n{}", rows.join("\n"))
				}
			}
			Err(err) => err,
		},
		"xp" => {
			let who = if target.is_empty() { uid.to_owned() } else { target.clone() };
			let rows = rank(rt, &p.id, 100);
			match rows.iter().find(|r| r["id"].as_str() == Some(who.as_str())) {
				Some(r) => format!(
					"{} está no nível {} com {} XP.",
					r["name"].as_str().unwrap_or("Membro"),
					r["level"],
					r["xp"]
				),
				None => format!("{target_name} ainda não tem XP."),
			}
		}
		"ranking" => {
			let rows = rank(rt, &p.id, 10);
			if rows.is_empty() {
				"Ninguém tem XP ainda.".into()
			} else {
				let lines: Vec<String> = rows
					.iter()
					.enumerate()
					.map(|(i, r)| {
						format!(
							"{}. {} — nível {} ({} XP)",
							i + 1,
							r["name"].as_str().unwrap_or("membro"),
							r["level"],
							r["xp"]
						)
					})
					.collect();
				format!("**Ranking do servidor**\n{}", lines.join("\n"))
			}
		}
		"aniversario" => {
			let who = if target.is_empty() { uid.to_owned() } else { target.clone() };
			let who_name = if target_name.is_empty() { who.clone() } else { target_name.clone() };
			let mut a = call(json!({"action":"birthdayAdd","date":opt_str(d,"data")}));
			a["target"] = json!(who);
			a["targetName"] = json!(who_name);
			match action(rt, &p.id, p, &a).await {
				Ok(_) => "Aniversário registrado.".into(),
				Err(err) => err,
			}
		}
		"vincular" => {
			let code = opt_str(d, "codigo");
			let mut store = read(rt, &link_key(&p.id));
			if !store.is_object() {
				store = json!({"pending":{},"links":[]});
			}
			let entry = store["pending"].get(&code.to_uppercase()).cloned();
			match entry {
				None => "Código inválido ou expirado. Gere outro código na tela do Discord.".into(),
				Some(e) => {
					store["pending"].as_object_mut().map(|m| m.remove(&code.to_uppercase()));
					let mut links = store["links"].as_array().cloned().unwrap_or_default();
					links.push(json!({"discordId": uid, "twitchId": e["twitchId"],
						"at": chrono::Utc::now().to_rfc3339()}));
					store["links"] = Value::Array(links);
					let _ = write(rt, &link_key(&p.id), &store);
					rt.emit("discord", json!({"profileId": p.id, "kind": "link"}));
					"Conta vinculada com sucesso. Suas ações passam a valer nas duas casas.".into()
				}
			}
		}
		"sorteio" => {
			let mut a = call(json!({"action":"giveawayStart","prize":opt_str(d,"premio")}));
			a["minutes"] = json!(opt_int(d, "minutos").unwrap_or(cfg["giveaway"]["minutes"].as_u64().unwrap_or(10)));
			a["winners"] = json!(opt_int(d, "vencedores").unwrap_or(1));
			match action(rt, &p.id, p, &a).await {
				Ok(_) => "Sorteio iniciado. Reaja com 🎉 para participar.".into(),
				Err(err) => err,
			}
		}
		_ => format!("Comando desconhecido: /{name}"),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn birthday_dates_accept_both_formats_and_reject_junk() {
		assert_eq!(parse_date("25/12"), Some((25, 12, None)));
		assert_eq!(parse_date("01/02/1990"), Some((1, 2, Some(1990))));
		assert_eq!(parse_date("25-12"), None);
		assert_eq!(parse_date("32/01"), None);
		assert_eq!(parse_date("01/13"), None);
		assert_eq!(parse_date(""), None);
	}

	#[test]
	fn level_curve_grows_quadratically() {
		assert_eq!(level(0), 0);
		assert_eq!(level(100), 1);
		assert_eq!(level(400), 2);
		assert_eq!(next_level_xp(0), 100);
		assert_eq!(next_level_xp(1), 400);
		assert!(next_level_xp(3) > next_level_xp(2));
	}

	#[test]
	fn slash_command_catalog_is_valid_for_the_discord_api() {
		let list = command_list();
		let cmds = list.as_array().expect("catalog is an array");
		assert!(cmds.len() <= 100);
		let mut names = vec![];
		for c in cmds {
			let name = c["name"].as_str().unwrap();
			assert!(!name.is_empty());
			assert!(c["description"].as_str().unwrap().len() <= 100);
			assert_eq!(c["type"], 1);
			assert!(!names.contains(&name), "comando duplicado: {name}");
			names.push(name);
		}
		for required in ["painel", "limpar", "ban", "mute", "warn", "sorteio", "vincular", "aniversario", "xp", "ranking"] {
			assert!(names.contains(&required), "falta o comando {required}");
		}
	}

	#[test]
	fn slash_permissions_cover_admin_and_moderators() {
		assert!(can_moderate(&json!({"permissions":"8"})));
		assert!(can_moderate(&json!({"permissions":"2"})));
		assert!(can_moderate(&json!({"permissions":"1099511627776"})));
		assert!(!can_moderate(&json!({"permissions":"0"})));
	}

	#[test]
	fn giveaway_draw_never_repeats_a_previous_winner() {
		let entries = vec![json!("1"), json!("2"), json!("3")];
		let first = pick_winners(&entries, 2, &[]);
		assert_eq!(first.len(), 2);
		let second = pick_winners(&entries, 2, &first);
		assert!(first.iter().all(|w| !second.contains(w)));
	}
}
