import {useState} from 'react';
import {LockKeyhole,ShieldCheck} from 'lucide-react';
import {api,errorText} from './api';
import {Card,Field} from './components';
export function Login({onLogin}:{onLogin:()=>void}){
 const [name,setName]=useState('owner');const [pin,setPin]=useState('');const [error,setError]=useState('');
 return <main className="login-screen"><Card title="Seu espaço está protegido"><div className="large-icon"><LockKeyhole/></div><p className="help">Entre com um acesso local do BotLive. Nenhum dado é enviado a um servidor.</p><form onSubmit={async e=>{e.preventDefault();try{await api('access.login',{name,pin});setPin('');onLogin()}catch(e){setError(errorText(e))}}}><Field label="Usuário local"><input autoFocus autoComplete="username" value={name} onChange={e=>setName(e.target.value)}/></Field><Field label="Senha local"><input type="password" autoComplete="current-password" value={pin} onChange={e=>setPin(e.target.value)}/></Field>{error&&<p className="inline-error">{error}</p>}<button className="primary">Abrir meu espaço</button></form></Card></main>
}
export function AccessSettings({notify,onLock}:{notify:(s:string)=>void;onLock:()=>void}){
 const [name,setName]=useState('owner');const [pin,setPin]=useState('');
 return <Card title="Permissões de edição"><p className="help">Crie primeiro a senha de “owner” (proprietário). Depois cadastre os moderadores e inclua seus nomes nos perfis que eles podem editar. Na próxima abertura, o app solicitará login local.</p><Field label="Nome do acesso local"><input value={name} onChange={e=>setName(e.target.value.toLowerCase())}/></Field><Field label="Senha (mínimo 6 caracteres)"><input type="password" autoComplete="new-password" value={pin} onChange={e=>setPin(e.target.value)}/></Field><div className="row"><button onClick={async()=>{try{await api('access.user',{name,pin});setPin('');notify('Acesso local salvo no cofre do sistema.')}catch(e){notify(errorText(e))}}}><ShieldCheck size={15}/>Salvar acesso</button><button onClick={async()=>{try{await api('access.lock');onLock()}catch(e){notify(errorText(e))}}}><LockKeyhole size={15}/>Bloquear painel</button></div></Card>
}
