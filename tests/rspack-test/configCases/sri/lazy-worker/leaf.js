export const value = 'leaf';

// Never called: jsdom has no Worker.
export const startWorker = () => new Worker(new URL('./worker.js', import.meta.url));
