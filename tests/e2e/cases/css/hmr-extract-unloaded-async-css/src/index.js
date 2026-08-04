document.getElementById('root').textContent = 'step-0';

window.loadFeature = () => import('./feature.js');

import.meta.webpackHot.accept();
