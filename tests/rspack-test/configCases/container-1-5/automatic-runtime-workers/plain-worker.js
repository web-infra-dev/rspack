import { parentPort } from 'worker_threads';

import('./local.js').then(({ default: value }) => {
  parentPort.postMessage({
    federation: Boolean(__webpack_require__.federation),
    value,
  });
});
