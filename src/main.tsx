import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles.css';
class Boundary extends React.Component<{children:React.ReactNode},{error:string}>{
 state={error:''};
 static getDerivedStateFromError(error:Error){return {error:error.message}}
 render(){return this.state.error?<main className="fatal"><h1>Vamos tentar de novo?</h1><p>A interface encontrou um problema. Seus dados salvos continuam no computador.</p><pre>{this.state.error}</pre><button onClick={()=>location.reload()}>Reabrir interface</button></main>:this.props.children}
}
ReactDOM.createRoot(document.getElementById('root')!).render(<React.StrictMode><Boundary><App/></Boundary></React.StrictMode>);
