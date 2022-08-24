import ReactDOM from 'react-dom';
import './styles.scss';
import HelloWorld from './components/hello-world';

function App() {
  return (
    <div className="app">
      <HelloWorld />
    </div>
  )
}

ReactDOM.render(<App />, document.getElementById('root'));
