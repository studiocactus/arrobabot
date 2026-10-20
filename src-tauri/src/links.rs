/// Endereços repetidos na mensagem que sai para o chat.
///
/// Cada resposta do bot entra na memória da conversa e a IA imita as falas
/// anteriores: o link fixo no fim da mensagem voltava dentro da própria
/// resposta da IA e se repetia a cada disparo (`:purple_heart: livepix.gg/x`
/// duas ou três vezes na mesma frase). O mesmo endereço sai **uma vez só** —
/// junto com o enfeite repetido que o precede (emoji, `:codigo:` e separadores).
/// Endereços diferentes continuam todos na mensagem: link não é problema,
/// repetição é.

/// O mesmo endereço, mesmo escrito com `https://` ou `www.`, conta como um só.
fn url_key(token: &str) -> Option<String> {
    let core = token.trim_matches(|c: char| matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | '…' | ')' | ']' | '}' | '\'' | '"' | '’'));
    let core = core.trim_start_matches(|c: char| matches!(c, '(' | '[' | '{' | '\'' | '"' | '“'));
    if core.is_empty() {
        return None;
    }
    let lower = core.to_lowercase();
    let bare = lower.strip_prefix("https://").or_else(|| lower.strip_prefix("http://")).unwrap_or(&lower);
    let bare = bare.strip_prefix("www.").unwrap_or(bare);
    let is_url = lower.starts_with("http://") || lower.starts_with("https://")
        || bare.split('/').next().is_some_and(|host| host.contains('.') && bare.contains('/'));
    if !is_url {
        return None;
    }
    Some(bare.to_owned())
}

/// De onde começa o enfeite antes do token: espaços, `:codigo:`, emojis e
/// separadores repetidos junto com o endereço saem juntos com ele.
fn decor_start(chars: &[char], mut start: usize) -> usize {
    loop {
        let mut t = start;
        while t > 0 && chars[t - 1].is_whitespace() {
            t -= 1;
        }
        if t > 0 && chars[t - 1] == ':' {
            let mut w = t - 1;
            while w > 0 && (chars[w - 1].is_alphanumeric() || chars[w - 1] == '_' || chars[w - 1] == '-') {
                w -= 1;
            }
            if w > 0 && chars[w - 1] == ':' && t - 1 > w {
                start = w - 1;
                continue;
            }
        }
        let c = chars[t - 1];
        if (!c.is_alphanumeric() && !c.is_ascii()) || matches!(c, '|' | '*' | '+' | '~' | '-' | '·') {
            start = t - 1;
            continue;
        }
        return t;
    }
}

/// Remove as repetições do mesmo endereço; o texto muda somente onde sobra
/// endereço duplicado.
pub fn dedupe(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut seen: Vec<String> = Vec::new();
    let mut drop: Vec<(usize, usize)> = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        let token: String = chars[start..i].iter().collect();
        let Some(key) = url_key(&token) else { continue };
        if seen.contains(&key) {
            drop.push((decor_start(&chars, start), i));
        } else {
            seen.push(key);
        }
    }
    let mut out = String::new();
    let mut at = 0usize;
    for (from, to) in drop {
        out.extend(chars[at..from].iter());
        at = to;
    }
    out.extend(chars[at..].iter());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O caso real: a IA repete o link com o enfeite; uma vez só sai.
    #[test]
    fn link_repetido_com_enfeite_sai_uma_vez() {
        let msg = "vlw pelo apoio mano, quem quiser dar aquela força na live cola aqui aksdkas :purple_heart: livepix.gg/thenees :purple_heart: livepix.gg/thenees";
        assert_eq!(dedupe(msg), "vlw pelo apoio mano, quem quiser dar aquela força na live cola aqui aksdkas :purple_heart: livepix.gg/thenees");
    }

    /// Com emoji no lugar do código, a mesma garantia.
    #[test]
    fn emoji_repetido_sai_junto_com_o_link() {
        let msg = "{{user}} apoie 💜 livepix.gg/thenees 💜 livepix.gg/thenees";
        assert_eq!(dedupe(msg), "{{user}} apoie 💜 livepix.gg/thenees");
    }

    /// Três repetições (o caso mais grave visto no chat) viram uma.
    #[test]
    fn tres_repeticoes_viram_uma() {
        let msg = "frase 💜 livepix.gg/thenees 💜 livepix.gg/thenees livepix.gg/thenees";
        assert_eq!(dedupe(msg), "frase 💜 livepix.gg/thenees");
    }

    /// Mesmo endereço com https:// e sem esquema é o mesmo endereço.
    #[test]
    fn esquema_www_e_caixa_nao_duplicam() {
        let msg = "apoie em livepix.gg/thenees e também em https://LivePix.gg/thenees";
        assert_eq!(dedupe(msg), "apoie em livepix.gg/thenees e também em");
    }

    /// Endereços diferentes continuam todos — link pode ser enviado.
    #[test]
    fn enderecos_diferentes_continuam_todos() {
        let msg = "canal twitch.tv/cactus e apoio livepix.gg/thenees";
        assert_eq!(dedupe(msg), msg);
    }

    /// Sem endereço repetido, o texto não muda nada.
    #[test]
    fn texto_sem_repeticao_fica_intacto() {
        let msg = "oi {{user}}, tudo bem? beba água! veja o edital (edital.md).";
        assert_eq!(dedupe(msg), msg);
    }

    /// Com palavras entre as repetições, só o endereço duplicado (e o enfeite
    /// colado nele) some; o resto da frase continua.
    #[test]
    fn palavras_entre_as_repeticoes_ficam() {
        let msg = "link livepix.gg/thenees — confira agora livepix.gg/thenees";
        assert_eq!(dedupe(msg), "link livepix.gg/thenees — confira agora");
    }
}
