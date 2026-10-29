/// Comandos naturais no chat e na fala (0.3, ampliado em 0.9): streamer e
/// moderadores falam ou escrevem por extenso e o bot executa o mesmo que
/// `!setgame`/`!settitle`, moderação e os comandos de voz. Parser
/// determinístico e offline, falha fechada: só entra no que o vocabulário
/// cobre; qualquer dúvida vira Ignore e a mensagem segue o fluxo normal.
/// Exige a palavra de ativação do Controle por voz, igual à fala.
#[derive(Debug,Clone,PartialEq)]
pub enum Talk{
 /// Não parece comando: segue o processamento normal do chat.
 Ignore,
 /// Intenção clara sem o valor: responde a dica de uso.
 Incomplete(&'static str),
 /// Categoria com alternativas na ordem, como "Minecraft ou AION2".
 Category(Vec<String>),
 /// Título novo, texto único.
 Title(String),
 /// Timeout com duração falada; None usa a duração configurada ("quicar" é
 /// sempre isto, nunca banimento).
 Timeout{login:String,seconds:Option<u64>},
 /// Banimento permanente: exige confirmação antes de executar.
 Ban{login:String},
 /// Volta o usuário banido ao canal.
 Unban{login:String},
 /// Confirma a ação permanente pendente dentro do prazo curto.
 Confirm,
 /// Cancela a confirmação pendente.
 Cancel,
 /// "Volte minha voz ao normal": devolve a voz no Voicemod ou na voz interna.
 VoiceNormal,
}
/// Verbos de troca: sem nenhum deles a frase não é comando de categoria/título.
const VERBS:&[&str]=&["troca","troque","trocar","trocando","muda","mude","mudar","mudando","altera","altere","alterar","alterando","coloca","coloque","colocar","bota","bote","botar","ponha","poe","define","defina","definir","seta","setar","mexe","mexer","ajusta","ajuste","ajustar","atualiza","atualize","atualizar","passa","passe","passar"];
/// O que identifica cada intenção; as duas no mesmo pedido não executam.
const CATEGORY:&[&str]=&["categoria","jogo","jogos","game","games"];
const TITLE:&[&str]=&["titulo","title"];
/// Moderação falada: "quicar" é apelido de timeout com a duração configurada.
const TIMEOUT:&[&str]=&["timeout","quicar","quica"];
const BAN:&[&str]=&["bana","bane","ban","banir","banimento"];
const UNBAN:&[&str]=&["desbana","desbane","desban","desbanir","unban","desbanimento"];
const CANCEL:&[&str]=&["cancele","cancela","cancelar","desiste","esquece"];
const CONFIRM:&[&str]=&["confirme","confirma","confirmar"];
const VOICE_VERBS:&[&str]=&["volta","volte","voltar","restaura","restaure","restaurar","normaliza","normalizar","devolve","devolver"];
const VOICE_NORM:&[&str]=&["normal","original","comum"];
/// Palavras que recusam o pedido: negação e pergunta não são ordem.
const NEGATIONS:&[&str]=&["nao","nem","nunca","jamais"];
const ASK_FIRST:&[&str]=&["que","qual","quais","quem","onde","quando","como","voce","sera"];
/// Preposições que apontam para o alvo: "no usuario123", "ao @fulano".
const PREPS:&[&str]=&["no","na","nos","nas","ao","aos","pro","pra","para","com","do","da","dos","das","de","em","o","a","os","as","um","uma","e"];
const HINT_ONE:&str="Fale um pedido por vez: repetindo só um comando de cada vez, eu executo direito.";
const HINT_TARGET:&str="Diga o usuário logo depois do comando, com @ ou o nome: dê timeout no @usuario123.";
/// O valor vem depois da primeira destas palavras após a intenção.
const SEPARATORS:&[&str]=&["para","pra","pro","por"];
/// Valor de pergunta, não de nome: silêncio, sem resposta no chat.
const QUESTIONS:&[&str]=&["que","qual","quais","quem","onde","quando","como","ai","aqui","ali","isso","esse","essa","aquilo","qualquer"];
/// Palavras de ligação que sobram nas bordas da categoria.
const EDGE_START:&[&str]=&["o","a","os","as","um","uma","do","da","de","dos","das","e","jogo","jogos","categoria"];
const EDGE_END:&[&str]=&["por","favor","agora","ai","de","do","da"];
const HINT_CATEGORY:&str="Para trocar a categoria, escreva o nome: mude a categoria para Minecraft. Dá para usar \"X ou Y\" e o bot tenta na ordem.";
const HINT_TITLE:&str="Para trocar o título, escreva o novo texto: mude o título para Ao vivo com a comunidade.";
/// Minúsculas e sem acento, um caractere por caractere: os índices valem no
/// texto original, então o valor sai da mensagem como foi escrita.
fn fold_char(c:char)->char{
 match c{
  'á'|'à'|'ã'|'â'|'ä'=>'a','é'|'è'|'ê'|'ë'=>'e','í'|'ì'|'î'|'ï'=>'i',
  'ó'|'ò'|'õ'|'ô'|'ö'=>'o','ú'|'ù'|'û'|'ü'=>'u','ç'=>'c',
  'Á'|'À'|'Ã'|'Â'|'Ä'=>'a','É'|'È'|'Ê'|'Ë'=>'e','Í'|'Ì'|'Î'|'Ï'=>'i',
  'Ó'|'Ò'|'Õ'|'Ô'|'Ö'=>'o','Ú'|'Ù'|'Û'|'Ü'=>'u','Ç'=>'c',
  other=>other.to_ascii_lowercase(),
 }
}
fn folded(s:&str)->String{s.chars().map(fold_char).collect()}
fn is_word(c:char)->bool{c.is_alphanumeric()||c=='_'||c=='-'}
/// Extensões [início,fim) das palavras em caracteres, para fatiar o original.
fn words(f:&[char])->Vec<(usize,usize)>{
 let mut out=Vec::new();let mut i=0;
 while i<f.len(){
  if is_word(f[i]){let start=i;while i<f.len()&&is_word(f[i]){i+=1}out.push((start,i));}
  else{i+=1}
 }
 out
}
fn at(chars:&[char],span:(usize,usize))->String{chars[span.0..span.1].iter().collect()}
fn has(list:&'static[&str],word:&str)->bool{list.contains(&word)}
fn clean_edges(s:&str)->String{
 s.trim().trim_matches(|c:char|matches!(c,'.'|','|'!'|'?'|';'|':'|'…'|'“'|'”'|'"'|'\'')).trim().to_owned()
}
/// Divide o valor pelo "ou" entre palavras: alternativas na ordem e a
/// primeira que existir na Twitch vale. Sem "ou" devolve o valor inteiro
/// como está; com mais de cinco nomes não parece alternativa e volta inteiro.
pub(crate) fn split_ou(value:&str)->Vec<String>{
 let chars:Vec<char>=value.chars().collect();
 let f:Vec<char>=folded(value).chars().collect();
 let mut pieces:Vec<String>=Vec::new();
 let mut last=0;let mut saw=false;let mut i=0;
 while i<chars.len(){
  if is_word(f[i]){
   let start=i;while i<chars.len()&&is_word(f[i]){i+=1}
   if f[start..i].iter().collect::<String>()=="ou"{
    saw=true;
    pieces.push(clean_edges(&chars[last..start].iter().collect::<String>()));
    last=i;
   }
  }else{i+=1}
 }
 if !saw{return vec![value.to_owned()]}
 pieces.push(clean_edges(&chars[last..].iter().collect::<String>()));
 pieces.retain(|p|!p.is_empty());
 if pieces.is_empty()||pieces.len()>5{return vec![clean_edges(value)]}
 pieces
}
/// Bordas da categoria: "para o jogo Minecraft" perde "o jogo" e fica
/// "Minecraft". A caixa original de cada palavra é mantida para a busca.
fn edge_trim(value:&str)->String{
 let chars:Vec<char>=value.chars().collect();
 let spans=words(&folded(value).chars().collect::<Vec<char>>());
 let word=|span|folded(&at(&chars,span));
 let mut start=0;let mut end=spans.len();
 while start<end&&has(EDGE_START,&word(spans[start])){start+=1}
 while end>start&&has(EDGE_END,&word(spans[end-1])){end-=1}
 spans[start..end].iter().map(|&sp|at(&chars,sp)).collect::<Vec<_>>().join(" ")
}
fn is_question(name:&str)->bool{
 let chars:Vec<char>=name.chars().collect();
 let spans=words(&folded(name).chars().collect::<Vec<char>>());
 spans.len()==1&&has(QUESTIONS,&folded(&at(&chars,spans[0])))
}
/// Remove o que está entre aspas: fala citada não é ordem minha.
fn strip_quotes(message:&str)->String{
 let mut out=String::new();let mut open=false;
 for c in message.chars(){
  if matches!(c,'"'|'\''|'«'|'»'|'“'|'”'|'„'){
   open=!open;out.push(' ');continue;
  }
  if open{out.push(' ')}else{out.push(c)}
 }
 out
}
/// Nome de usuário da Twitch: letras, número, hífen e sublinhado, 3 a 25.
fn login_ok(w:&str)->bool{w.len()>=3&&w.len()<=25&&w.chars().all(|c|c.is_ascii_alphanumeric()||c=='_'||c=='-')}
/// Número por extenso ou dígito: "cinco", "vinte", "30".
fn number(w:&str)->Option<u64>{
 if let Ok(n)=w.parse::<u64>(){return Some(n)}
 const W:&[(&str,u64)]=&[("zero",0),("um",1),("uma",1),("dois",2),("duas",2),("tres",3),("quatro",4),("cinco",5),("seis",6),("sete",7),("oito",8),("nove",9),("dez",10),("onze",11),("doze",12),("treze",13),("catorze",14),("quinze",15),("dezesseis",16),("dezessete",17),("dezoito",18),("dezenove",19),("vinte",20),("trinta",30),("quarenta",40),("cinquenta",50),("sessenta",60)];
 W.iter().find(|(k,_)|*k==w).map(|(_,v)|*v)
}
const UNIT_MIN:&[&str]=&["minuto","minutos","min"];
const UNIT_SEC:&[&str]=&["segundo","segundos","seg"];
const UNIT_HR:&[&str]=&["hora","horas"];
/// Soma tempos falados depois do verbo: "cinco minutos", "um minuto e trinta
/// segundos", "30" sozinho vale segundos. Número colado em nome não conta.
fn duration_after(tokens:&[String],start:usize)->Option<u64>{
 let mut total:u64=0;let mut found=false;let mut i=start;
 while i<tokens.len(){
  let Some(n)=number(&tokens[i]) else{i+=1;continue};
  let unit=tokens.get(i+1).map(String::as_str).unwrap_or("");
  let step=if has(UNIT_MIN,unit){total=total.saturating_add(n*60);2}
   else if has(UNIT_SEC,unit){total=total.saturating_add(n);2}
   else if has(UNIT_HR,unit){total=total.saturating_add(n*3600);2}
   else{total=total.saturating_add(n);1};
  found=true;i+=step;
 }
 if found{Some(total.clamp(1,86400))}else{None}
}
/// Acha o login do alvo depois do verbo: @em qualquer lugar, o nome depois de
/// preposição ou a palavra logo em seguida. Sem chute: quem não aparece não
/// é punido.
fn target_login(tokens:&[String],after:usize)->Option<String>{
 for t in tokens.iter().skip(after){
  if let Some(l)=t.strip_prefix('@'){
   if login_ok(l){return Some(l.to_lowercase())}
  }
 }
 for (i,t) in tokens.iter().enumerate().skip(after){
  if has(PREPS,t){
   if let Some(n)=tokens.get(i+1){
    if login_ok(n)&&number(n).is_none(){return Some(n.to_lowercase())}
   }
  }
 }
 tokens.get(after).filter(|t|login_ok(t)&&number(t).is_none()).map(|t|t.to_lowercase())
}
/// Famílias de intenção além de categoria/título: moderação falada,
/// confirmação, cancelamento e "volte minha voz ao normal". Devolve None
/// quando nenhuma família aparece (aí o parser segue para categoria/título).
fn moderation(tokens:&[String])->Option<Talk>{
 let mut timeouts:Vec<usize>=Vec::new();let mut bans:Vec<usize>=Vec::new();let mut unbans:Vec<usize>=Vec::new();
 let mut cancel:Option<usize>=None;let mut confirm:Option<usize>=None;let mut voice:Option<usize>=None;
 let mut i=0;
 while i<tokens.len(){
  let w=tokens[i].as_str();
  if has(TIMEOUT,w)||(w=="time"&&tokens.get(i+1).is_some_and(|n|n=="out")){timeouts.push(i)}
  else if has(BAN,w){bans.push(i)}
  else if has(UNBAN,w){unbans.push(i)}
  else if has(CANCEL,w){cancel=Some(i)}
  else if has(CONFIRM,w){confirm=Some(i)}
  else if has(VOICE_VERBS,w)&&tokens.iter().any(|t|t=="voz")&&tokens.iter().any(|t|has(VOICE_NORM,t)){voice=Some(i)}
  i+=1;
 }
 let mut families=0;
 if !timeouts.is_empty(){families+=1}
 if !bans.is_empty(){families+=1}
 if !unbans.is_empty(){families+=1}
 if cancel.is_some(){families+=1}
 if confirm.is_some(){families+=1}
 if voice.is_some(){families+=1}
 if families==0{return None}
 // "Confirme o banimento" é uma só intenção: a confirmação do ban pendente.
 let confirm_ban=confirm.is_some()&&bans.len()==1&&families==2;
 if confirm_ban{return Some(Talk::Confirm)}
 if families>1{return Some(Talk::Incomplete(HINT_ONE))}
 // O primeiro verbo da frase: negação ou pergunta antes dele não é ordem.
 let mut actions:Vec<usize>=timeouts.iter().chain(bans.iter()).chain(unbans.iter()).copied().collect();
 for extra in [cancel,confirm,voice].into_iter().flatten(){actions.push(extra)}
 let first=*actions.iter().min().unwrap();
 if tokens[..first].iter().any(|w|has(NEGATIONS,w)||has(ASK_FIRST,w)){return Some(Talk::Ignore)}
 if cancel.is_some(){return Some(Talk::Cancel)}
 if confirm.is_some(){return Some(Talk::Confirm)}
 if voice.is_some(){
  return Some(Talk::VoiceNormal)
}
 let action=timeouts.first().copied().or_else(||bans.first().copied()).or_else(||unbans.first().copied()).unwrap_or(0);
 let login=match target_login(tokens,action+1){Some(l)=>l,None=>return Some(Talk::Incomplete(HINT_TARGET))};
 if let Some(&idx)=timeouts.first(){
  Some(Talk::Timeout{login,seconds:duration_after(tokens,idx+1)})
 }else if unbans.first().is_some(){
  Some(Talk::Unban{login})
 }else{
  Some(Talk::Ban{login})
 }
}
/// Lê uma mensagem de chat como comando natural. A ordem é sempre a mesma:
/// ativação, verbo, intenção, valor; o que não passa em uma etapa cai em
/// Ignore e a mensagem segue o fluxo normal do chat.
pub fn parse(message:&str,activation:&str)->Talk{
 // Sem palavra cadastrada não há comando natural no chat.
 let stops:Vec<String>=activation.split(',').map(str::trim).filter(|s|!s.is_empty()).map(|s|folded(s)).collect();
 if stops.is_empty(){return Talk::Ignore}
 // Aspas: o que o streamer citou de alguém não é ordem minha.
 let message=strip_quotes(message);
 let orig:Vec<char>=message.chars().collect();
 let f:Vec<char>=orig.iter().map(|&c|fold_char(c)).collect();
 let mentioned=stops.iter().any(|stop|{
  let needle:Vec<char>=stop.chars().collect();
  !needle.is_empty()&&f.windows(needle.len()).any(|w|w==needle.as_slice())
 });
 if !mentioned{return Talk::Ignore}
 let spans=words(&f);
 let tokens:Vec<String>=spans.iter().map(|&sp|at(&f,sp)).collect();
 // Moderação falada, confirmação, cancelamento e voz normal entram antes:
 // não dependem dos verbos de troca de categoria/título.
 if let Some(talk)=moderation(&tokens){
  // Pedido misturado com categoria/título não executa pela metade.
  if !matches!(talk,Talk::Ignore)&&(tokens.iter().any(|w|has(CATEGORY,w))||tokens.iter().any(|w|has(TITLE,w))){
   return Talk::Incomplete(HINT_ONE)
  }
  return talk
 }
 if !tokens.iter().any(|w|has(VERBS,w)){return Talk::Ignore}
 let Some(kw_idx)=tokens.iter().position(|w|has(CATEGORY,w)||has(TITLE,w)) else{return Talk::Ignore};
 let in_category=tokens.iter().any(|w|has(CATEGORY,w));
 let in_title=tokens.iter().any(|w|has(TITLE,w));
 if in_category==in_title{return Talk::Ignore}// nem um, ou os dois no mesmo pedido
 let is_title=in_title;
 let Some(sep)=spans.iter().enumerate().find(|(i,_)|*i>kw_idx&&has(SEPARATORS,&tokens[*i])).map(|(_,sp)|*sp)
 else{return Talk::Incomplete(if is_title{HINT_TITLE}else{HINT_CATEGORY})};
 let value:String=orig[sep.1..].iter().collect();
 let vf:Vec<char>=folded(&value).chars().collect();
 if words(&vf).is_empty(){return Talk::Incomplete(if is_title{HINT_TITLE}else{HINT_CATEGORY})}
 if is_title{
  if is_question(value.trim()){return Talk::Ignore}
  return Talk::Title(value.trim().to_owned())
 }
 let names:Vec<String>=split_ou(&value).into_iter().map(|piece|edge_trim(&piece)).filter(|n|!n.is_empty()).collect();
 if names.is_empty()||names.iter().any(|n|matches!(folded(n).as_str(),"mim"|"mim por favor")){return Talk::Incomplete(HINT_CATEGORY)}
 if is_question(&names[0]){return Talk::Ignore}
 if names.iter().any(|n|n.chars().count()>120){return Talk::Ignore}
 Talk::Category(names)
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn pedido_de_categoria_pra_mim_ainda_precisa_do_jogo(){
  assert_eq!(parse("arroba, altera a categoria do jogo pra mim","arroba"),Talk::Incomplete(HINT_CATEGORY));
  assert_eq!(parse("arroba, altera a categoria pra mim por favor","arroba"),Talk::Incomplete(HINT_CATEGORY));
 }
 #[test] fn o_exemplo_completo_vira_categoria_com_plano_b(){
  assert_eq!(parse("Arroba, altere a categoria do jogo para Minecraft ou AION2","arroba"),Talk::Category(vec!["Minecraft".into(),"AION2".into()]));
 }
 #[test] fn intenções_comuns_são_lidas_com_e_sem_acento(){
  assert_eq!(parse("arroba, mude o jogo pra valorant","arroba"),Talk::Category(vec!["valorant".into()]));
  assert_eq!(parse("ÁRROBA! muda o titulo para Ranked com viewers","arroba"),Talk::Title("Ranked com viewers".into()));
  assert_eq!(parse("arroba, troque a categoria por Counter-Strike 2","arroba"),Talk::Category(vec!["Counter-Strike 2".into()]));
 }
 #[test] fn sem_ativacao_a_mensagem_e_chat_normal(){
  assert_eq!(parse("mude a categoria para Minecraft","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, mude a categoria para Minecraft",""),Talk::Ignore);
  assert_eq!(parse("arroba, mude a categoria para Minecraft","botlive"),Talk::Ignore);
 }
 #[test] fn sem_verbo_ou_com_as_duas_intencoes_nao_executa(){
  assert_eq!(parse("arroba, Minecraft é o melhor jogo","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, mude o jogo e o título para X","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, você muda o jogo?","arroba"),Talk::Incomplete(HINT_CATEGORY));
 }
 #[test] fn intencao_clara_sem_nome_responde_o_modo_de_usar(){
  assert_eq!(parse("arroba, mude a categoria","arroba"),Talk::Incomplete(HINT_CATEGORY));
  assert_eq!(parse("arroba, mude o título para","arroba"),Talk::Incomplete(HINT_TITLE));
  assert_eq!(parse("arroba, mude o jogo para ...","arroba"),Talk::Incomplete(HINT_CATEGORY));
 }
 #[test] fn pergunta_de_valor_fica_em_silencio(){
  assert_eq!(parse("arroba, o jogo muda para quê?","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, mude o título para quê?","arroba"),Talk::Ignore);
 }
 #[test] fn o_titulo_guarda_pontuacao_e_o_titulo_com_ou_nao_vira_alternativa(){
  assert_eq!(parse("arroba, mude o título para Ao vivo às 21h!","arroba"),Talk::Title("Ao vivo às 21h!".into()));
  assert_eq!(parse("arroba, mude o título para RPG ou horror","arroba"),Talk::Title("RPG ou horror".into()));
 }
 #[test] fn a_categoria_tira_palavras_de_ligacao_e_pontuacao_das_bordas(){
  assert_eq!(parse("arroba, por favor mude a categoria para o jogo Minecraft!","arroba"),Talk::Category(vec!["Minecraft".into()]));
  assert_eq!(parse("arroba, altere a categoria do jogo para MINECRAFT","arroba"),Talk::Category(vec!["MINECRAFT".into()]));
 }
 #[test] fn alternativas_da_twitch_comecam_pelo_primeiro_nome(){
  assert_eq!(split_ou("Minecraft ou AION2"),vec!["Minecraft".to_owned(),"AION2".to_owned()]);
  assert_eq!(split_ou("Minecraft OU AION2"),vec!["Minecraft".to_owned(),"AION2".to_owned()]);
  assert_eq!(split_ou("Counter-Strike 2"),vec!["Counter-Strike 2".to_owned()]);
  assert_eq!(split_ou("RPG ou"),vec!["RPG".to_owned()]);
  assert_eq!(split_ou("a ou b ou c ou d ou e ou f"),vec!["a ou b ou c ou d ou e ou f".to_owned()]);
 }
 #[test] fn timeout_falado_com_duracao_por_extenso(){
  assert_eq!(parse("Arroba, dê timeout no usuario123 por cinco minutos.","arroba"),Talk::Timeout{login:"usuario123".into(),seconds:Some(300)});
  assert_eq!(parse("arroba, timeout de 30 segundos no troll2","arroba"),Talk::Timeout{login:"troll2".into(),seconds:Some(30)});
  assert_eq!(parse("arroba, timeout no usuario123","arroba"),Talk::Timeout{login:"usuario123".into(),seconds:None});
  assert_eq!(parse("arroba, dê um time out no @Silva_90 por um minuto e trinta segundos","arroba"),Talk::Timeout{login:"silva_90".into(),seconds:Some(90)});
 }
 #[test] fn quicar_e_sempre_timeout_nunca_banimento(){
  assert_eq!(parse("Arroba, quicar o usuario123.","arroba"),Talk::Timeout{login:"usuario123".into(),seconds:None});
  assert_eq!(parse("arroba, quica o usuario123 por dez minutos","arroba"),Talk::Timeout{login:"usuario123".into(),seconds:Some(600)});
  assert_ne!(parse("arroba, quicar o usuario123","arroba"),Talk::Ban{login:"usuario123".into()});
 }
 #[test] fn ban_e_desban_pedem_o_usuario_claro(){
  assert_eq!(parse("Arroba, bana o usuario123.","arroba"),Talk::Ban{login:"usuario123".into()});
  assert_eq!(parse("arroba, ban @Trol_2","arroba"),Talk::Ban{login:"trol_2".into()});
  assert_eq!(parse("Arroba, desbana o usuario123.","arroba"),Talk::Unban{login:"usuario123".into()});
  assert_eq!(parse("arroba, unban usuario123","arroba"),Talk::Unban{login:"usuario123".into()});
 }
 #[test] fn alvo_ausente_pede_o_nome_em_ves_de_chutar(){
  assert_eq!(parse("arroba, dê timeout","arroba"),Talk::Incomplete(HINT_TARGET));
  assert_eq!(parse("arroba, bana o","arroba"),Talk::Incomplete(HINT_TARGET));
  assert!(matches!(parse("arroba, bana o impostor","arroba"),Talk::Incomplete(_))||matches!(parse("arroba, bana o impostor","arroba"),Talk::Ban{..}));
  assert!(matches!(parse("arroba, timeout no user!","arroba"),Talk::Timeout{..}));
 }
 #[test] fn negacao_e_pergunta_nao_sao_ordem(){
  assert_eq!(parse("arroba, não bana o usuario123","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, você bana o usuario123?","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, quem ele baniu?","arroba"),Talk::Ignore);
 }
 #[test] fn fala_citada_nao_vira_ordem(){
  assert_eq!(parse("arroba, ele disse \"bana o usuario123\"","arroba"),Talk::Ignore);
  assert_eq!(parse("arroba, chat falou 'dê timeout no amiguinho'","arroba"),Talk::Ignore);
 }
 #[test] fn dois_pedidos_diferentes_nao_executam_parcialmente(){
  assert_eq!(parse("arroba, mude a categoria para Minecraft e bana o usuario123","arroba"),Talk::Incomplete(HINT_ONE));
  assert_eq!(parse("arroba, bana o usuario123 e desbana o outro","arroba"),Talk::Incomplete(HINT_ONE));
  assert_eq!(parse("arroba, cancele e mude o título para X","arroba"),Talk::Incomplete(HINT_ONE));
 }
 #[test] fn cancelar_e_confirmar_sao_intencoes_proprias(){
  assert_eq!(parse("Arroba, cancele.","arroba"),Talk::Cancel);
  assert_eq!(parse("arroba, cancela","arroba"),Talk::Cancel);
  assert_eq!(parse("arroba, confirme","arroba"),Talk::Confirm);
  assert_eq!(parse("arroba, confirme o banimento","arroba"),Talk::Confirm);
  assert_eq!(parse("arroba, confirme o ban","arroba"),Talk::Confirm);
 }
 #[test] fn voltar_a_voz_normal_e_intencao_propria(){
  assert_eq!(parse("Arroba, volte minha voz ao normal.","arroba"),Talk::VoiceNormal);
  assert_eq!(parse("arroba, volta a voz pro normal","arroba"),Talk::VoiceNormal);
  assert_eq!(parse("arroba, restaure minha voz original","arroba"),Talk::VoiceNormal);
  assert_eq!(parse("arroba, devolve minha voz ao normal","arroba"),Talk::VoiceNormal);
 }
 #[test] fn numeracao_por_extenso_e_digitos_contam_igual(){
  assert_eq!(duration_after(&tokens_of("por vinte minutos"),0),Some(1200));
  assert_eq!(duration_after(&tokens_of("de 45 segundos"),0),Some(45));
  assert_eq!(duration_after(&tokens_of("por duas horas"),0),Some(7200));
  assert_eq!(duration_after(&tokens_of("por cinco"),0),Some(5),"sem unidade vale segundos");
  assert_eq!(duration_after(&tokens_of("por nada"),0),None);
 }
 fn tokens_of(s:&str)->Vec<String>{
  let f:Vec<char>=folded(s).chars().collect();
  words(&f).iter().map(|&sp|at(&f,sp)).collect()
 }
 #[test] fn login_colado_em_numero_nao_vira_duracao(){
  // "usuario123" tem dígitos, mas não é um número: o alvo vale e não conta segundos.
  assert_eq!(parse("arroba, timeout no usuario123","arroba"),Talk::Timeout{login:"usuario123".into(),seconds:None});
 }
}