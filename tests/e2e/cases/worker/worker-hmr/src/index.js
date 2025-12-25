const worker = new Worker(new URL('./worker.js', import.meta.url));

const root = document.getElementById('root');
root.textContent = 'Main thread loaded';

const workerResult = document.createElement('div');
workerResult.id = 'worker-result';
root.appendChild(workerResult);

worker.addEventListener('message', (e) => {
  workerResult.textContent = `Worker: ${e.data}`;
});
