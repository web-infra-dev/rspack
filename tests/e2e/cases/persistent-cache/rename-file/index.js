import a from './a.js';
import b from './b.js';

const div = document.createElement('div');
div.innerText = a + b;
document.body.appendChild(div);
