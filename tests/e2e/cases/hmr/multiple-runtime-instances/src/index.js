import { value } from './value.js';

const div = document.createElement('div');
div.innerHTML = value
div.setAttribute('data-testid', document.querySelectorAll('div').length.toString());
document.body.appendChild(div);
