import { parentPort, Worker } from 'worker_threads';

const child = new Worker(new URL('./plain-worker.js', import.meta.url), { name: 'plain-worker' });
const childResult = new Promise((resolve, reject) => {
  child.on('message', resolve);
  child.on('error', reject);
});

Promise.all([import('remote/value'), childResult]).then(async ([{ default: value }, result]) => {
  await child.terminate();
  parentPort.postMessage({
    federation: Boolean(__webpack_require__.federation.instance),
    value,
    child: result,
  });
});
