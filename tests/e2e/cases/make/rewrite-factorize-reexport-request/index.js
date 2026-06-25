import { value } from './reexport.js';

const div = document.createElement('div');
div.innerText = String(value);
document.body.appendChild(div);
