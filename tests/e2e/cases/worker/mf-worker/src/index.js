// Create a worker that uses a shared library
const worker = new Worker(new URL('./worker.js', import.meta.url));

const root = document.getElementById('root');
root.textContent = 'Main thread loaded';

const workerResult = document.createElement('div');
workerResult.id = 'worker-result';
root.appendChild(workerResult);

// Listen for messages from worker
worker.addEventListener('message', (e) => {
  workerResult.textContent = `Worker: ${e.data}`;
});

// Listen for errors from worker
worker.addEventListener('error', (e) => {
  workerResult.textContent = `Error: ${e.message}`;
  console.error('Worker error:', e);
});

// Send message to worker to trigger it
worker.postMessage('start');
