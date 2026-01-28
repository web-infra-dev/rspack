// @ts-nocheck
import { a } from './missing-file-1.js';
import { b } from './missing-file-2.js';

document.getElementById('root').innerHTML = `
<span id="missing-file-1">${a}</span>
<span id="missing-file-2">${b}</span>
`;
