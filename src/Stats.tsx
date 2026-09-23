import {useEffect,useState} from 'react';
import {BarChart3,Activity,Users,Zap} from 'lucide-react';
import {api,errorText} from './api';
import {Card,Empty} from './components';
type StatsData={messages:number;actions:number;followers:number;hours:{hour:string;count:number}[];commands:{name:string;count:number}[]};
export default function Stats({profileId,notify}:{profileId:string;notify:(s:string)=>void}){
 const [data,setData]=useState<StatsData>({messages:0,actions:0,followers:0,hours:[],commands:[]});
 useEffect(()=>{api<StatsData>('stats',{profileId}).then(setData).catch(e=>notify(errorText(e)))},[profileId]);
 const peak=Math.max(0,...data.hours.map(h=>h.count));
 return <><div className="metrics">{[[Activity,'Mensagens de chat',data.messages],[Zap,'Ações concluídas',data.actions],[Users,'Novos seguidores',data.followers],[BarChart3,'Pico de mensagens / hora',peak]].map(([Icon,label,value])=>{const I=Icon as typeof Activity;return <Card className="metric" key={String(label)}><div><span>{String(label)}</span><I size={18}/></div><strong>{String(value)}</strong><small>Histórico local retido</small></Card>})}</div><div className="two-columns"><Card title="Ritmo do chat"><p className="help">Últimas 24 horas com atividade registrada, em UTC.</p>{data.hours.slice().reverse().map(h=><div className="chart-row" key={h.hour}><span>{h.hour.slice(5).replace('T',' ')}</span><div><i style={{width:h.count/Math.max(peak,1)*100+'%'}}/></div><strong>{h.count}</strong></div>)}{!data.hours.length&&<Empty title="A próxima conversa começa o gráfico">Conecte um canal para registrar a atividade.</Empty>}</Card><Card title="Automações mais usadas">{data.commands.map(c=><div className="list-row" key={c.name}><span>{c.name}</span><strong>{c.count}</strong></div>)}{!data.commands.length&&<p className="help">Os fluxos executados aparecerão aqui.</p>}</Card></div><p className="help">As estatísticas usam o histórico local, limitado aos 10 mil registros mais recentes do aplicativo. Não representam métricas de audiência da plataforma.</p></>
}
