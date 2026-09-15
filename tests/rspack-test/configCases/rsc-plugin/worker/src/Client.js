"use client";

export const Client = () => {
  const createWorker = () => {
    new Worker(
      /* webpackChunkName: "my-worker" */ new URL(
        './worker.js',
        import.meta.url,
      ),
      { type: 'module' },
    );
  };

  return <button onClick={createWorker}>spawn worker</button>;
};
