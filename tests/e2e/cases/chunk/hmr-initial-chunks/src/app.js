import './shared.css';
import { value, instance } from './shared.js';

function render() {
  document.querySelector('#root').textContent = value;
  window.sharedInstance = instance;
}
render();

if (import.meta.webpackHot) {
  import.meta.webpackHot.accept('./shared.js', render);
  import.meta.webpackHot.accept();
}
