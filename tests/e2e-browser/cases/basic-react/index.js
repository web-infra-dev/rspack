console.log('[e2e] crossOriginIsolated:', window.crossOriginIsolated);
console.log(
  '[e2e] SharedArrayBuffer available:',
  typeof SharedArrayBuffer !== 'undefined',
);
console.log('[e2e] starting rspack browser build');

new Promise((resolve, reject) => {
  const worker = new Worker(new URL('./rspack.worker.js', import.meta.url), {
    type: 'module',
  });

  worker.addEventListener('message', ({ data }) => {
    if (data.type === 'error') {
      reject(new Error(data.error));
      worker.terminate();
      return;
    }

    const outputDOM = document.createElement('div');
    outputDOM.id = 'output';
    outputDOM.innerHTML = data.output;
    document.body.appendChild(outputDOM);
    worker.terminate();
    resolve();
  });

  worker.addEventListener('error', (event) => {
    reject(event.error || new Error(event.message));
    worker.terminate();
  });
});
