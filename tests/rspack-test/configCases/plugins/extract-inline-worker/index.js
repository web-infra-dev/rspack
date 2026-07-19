new Worker(
  URL.createObjectURL(
    new Blob(
      [
        'self.onmessage = event => postMessage("worker-result-marker:" + event.data);\n',
      ],
      { type: 'application/javascript' },
    ),
  ),
);
