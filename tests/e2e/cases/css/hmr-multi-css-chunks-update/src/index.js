import('./a.js');
import('./b.js');

document.getElementById('root').textContent = 'step-0';

import.meta.webpackHot.accept();
