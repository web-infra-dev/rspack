const worker = new Worker(new URL("./worker.js", import.meta.url));
module.exports = { hasWorker: !!worker };
