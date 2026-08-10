const jsUrl = new URL('./target.js', import.meta.url);

const loadScript = (url) =>
  new Promise((resolve, reject) => {
    const script = document.createElement('script');
    script.onload = resolve;
    script.onerror = reject;
    script.src = url;
    document.head.appendChild(script);
  });

const runWorker = (url) =>
  new Promise((resolve, reject) => {
    const worker = new Worker(url);
    worker.onmessage = async ({ data }) => {
      try {
        await worker.terminate();
        resolve(data);
      } catch (error) {
        reject(error);
      }
    };
    worker.postMessage('run');
  });

it('should execute a new URL JavaScript entry in a script element and worker', async () => {
  await loadScript(jsUrl.href);
  expect(globalThis.NEW_URL_SCRIPT_EXECUTED).toBe(true);
  expect(await runWorker(jsUrl)).toBe('executed in worker');
});
