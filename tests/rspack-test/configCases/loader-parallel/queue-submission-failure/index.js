it('should continue the bundled worker queue after consecutive submission failures', async () => {
  const { Tinypool } = await loadTinypool();
  const pool = new Tinypool({
    filename: workerFilename,
    minThreads: 1,
    maxThreads: 1,
    useAtomics: false,
  });
  let timeout;
  try {
    const buffer = new Int32Array(new SharedArrayBuffer(4));
    const results = [
      pool.run(buffer),
      pool.run(() => {}).catch(error => error.name),
      pool.run(() => {}).catch(error => error.name),
      pool.run(buffer),
    ];
    expect(pool.queueSize).toBe(3);

    Atomics.store(buffer, 0, 1);
    Atomics.notify(buffer, 0);

    const completed = await Promise.race([
      Promise.all(results),
      new Promise((_, reject) => {
        timeout = setTimeout(() => reject(new Error('Worker queue stalled')), 5000);
      }),
    ]);
    expect(completed).toEqual(['ok', 'DataCloneError', 'DataCloneError', 'ok']);
    expect(pool.queueSize).toBe(0);
  } finally {
    clearTimeout(timeout);
    await pool.destroy();
  }
});
