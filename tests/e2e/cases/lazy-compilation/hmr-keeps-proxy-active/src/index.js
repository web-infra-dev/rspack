import { name } from './sibling.js';

document.body.dataset.sibling = name;

Promise.all([
  import('./lazy-a.js'),
  import('./lazy-b.js'),
  import('./lazy-c.js'),
  import('./lazy-d.js'),
]).then((all) => {
  document.body.dataset.lazy = all.map((m) => m.text).join(',');
});
