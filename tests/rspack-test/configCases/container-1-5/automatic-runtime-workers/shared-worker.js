import { parentPort } from 'worker_threads';

import('./shared-async.js').then(({ default: value }) => {
  parentPort.postMessage({
    federation: Boolean(__webpack_require__.federation.instance),
    value,
  });
});
