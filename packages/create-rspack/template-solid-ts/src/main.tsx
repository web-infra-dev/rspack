import { render } from '@solidjs/web';
import App from './App';
import './index.css';

const root = document.getElementById('root');
if (root) {
  render(() => <App />, root);
}
