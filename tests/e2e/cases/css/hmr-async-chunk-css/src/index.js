import(/* webpackChunkName: "feature" */ './feature.js').then(() => {
  document.getElementById('root').textContent = 'feature loaded';
});

import.meta.webpackHot.accept();
