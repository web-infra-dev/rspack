import { Readable } from 'node:stream';
import rspack from '@rspack/core';
import express from 'express';
import path from 'path';
import webpackDevMiddleware from 'webpack-dev-middleware';
import webpackHotMiddleware from 'webpack-hot-middleware';
import { Worker } from 'worker_threads';

import rspackConfig, { rscState } from './rspack.config.js';

const SERVER_PORT = 1716;
const WORKER_PORT = 3016;

let hotMiddleware;
let currentWorker;
let workerRestartPromise;

const compiler = rspack(rspackConfig);

compiler.compilers[1].hooks.done.tapPromise('RestartWorker', async (stats) => {
  if (stats.hasErrors()) {
    console.error('[Server] Build failed with errors');
    return;
  }

  workerRestartPromise = (async () => {
    if (currentWorker) {
      await currentWorker.terminate();
      currentWorker = null;
    }

    currentWorker = await createServerWorker();
    if (rscState.onServerComponentChanged) {
      hotMiddleware.publish({ type: 'rsc:update' });
    }
    rscState.onServerComponentChanged = false;
  })();

  await workerRestartPromise;
});

compiler.compilers[0].hooks.done.tapPromise('WaitForWorker', async () => {
  if (!workerRestartPromise) return;
  try {
    await workerRestartPromise;
    await new Promise((resolve) => setTimeout(resolve, 1000));
  } catch (_error) {
    // no-op in dev server orchestration
  }
});

const app = express();

app.use(
  webpackDevMiddleware(compiler, {
    writeToDisk: true,
  }),
);

hotMiddleware = webpackHotMiddleware(compiler.compilers[0], {
  log: console.log,
  path: '/__rspack_hmr',
  heartbeat: 10 * 1000,
});
app.use(hotMiddleware);

app.use(async (req, res, next) => {
  if (req.path.startsWith('/__rspack_hmr')) {
    next();
    return;
  }

  if (!currentWorker) {
    res.status(503).send('RSC worker is not ready');
    return;
  }

  const requestUrl = `http://127.0.0.1:${WORKER_PORT}${req.originalUrl}`;
  const headers = new Headers();
  for (const [name, value] of Object.entries(req.headers)) {
    if (value == null || name === 'host' || name === 'content-length') {
      continue;
    }
    if (Array.isArray(value)) {
      for (const item of value) {
        headers.append(name, item);
      }
    } else {
      headers.set(name, value);
    }
  }

  const hasBody = req.method !== 'GET' && req.method !== 'HEAD';
  const response = await fetch(requestUrl, {
    method: req.method,
    headers,
    body: hasBody ? req : undefined,
    duplex: hasBody ? 'half' : undefined,
  });
  res.status(response.status);
  response.headers.forEach((value, name) => {
    if (name === 'transfer-encoding' || name === 'content-encoding') {
      return;
    }
    res.setHeader(name, value);
  });

  if (!response.body) {
    res.end();
    return;
  }
  Readable.fromWeb(response.body).pipe(res);
});

function createServerWorker() {
  return new Promise((resolve, reject) => {
    const workerPath = path.join(import.meta.dirname, 'dist/main.cjs');
    const worker = new Worker(workerPath, {
      env: {
        ...process.env,
        RSC_WORKER_PORT: String(WORKER_PORT),
      },
    });

    worker.on('message', (message) => {
      if (message.type === 'ready') {
        resolve(worker);
      }
    });

    worker.on('error', (error) => {
      reject(error);
    });

    worker.on('exit', (code) => {
      if (code !== 0) {
        reject(new Error(`Worker stopped with exit code ${code}`));
      }
    });

    setTimeout(() => {
      reject(new Error('Worker initialization timeout'));
    }, 10000);
  });
}

const server = app.listen(SERVER_PORT, 'localhost', function () {
  console.log('Host dev server is running on %j', server.address());
});
