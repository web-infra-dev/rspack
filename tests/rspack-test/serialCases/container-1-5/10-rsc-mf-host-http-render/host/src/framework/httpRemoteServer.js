import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import path from 'node:path';

let server;

function getContentType(filePath) {
  if (filePath.endsWith('.js')) return 'text/javascript; charset=utf-8';
  if (filePath.endsWith('.mjs')) return 'text/javascript; charset=utf-8';
  if (filePath.endsWith('.json')) return 'application/json; charset=utf-8';
  return 'application/octet-stream';
}

function resolveSafePath(rootDirectory, requestPath) {
  const normalized = path.posix.normalize(`/${requestPath || ''}`);
  const relativePath = normalized.replace(/^\/+/, '');
  const fullPath = path.resolve(rootDirectory, relativePath);
  if (!fullPath.startsWith(rootDirectory)) {
    return null;
  }
  return fullPath;
}

export async function ensureRemoteServer(port) {
  if (server) {
    return;
  }
  const rootDirectory = path.resolve(__dirname, '..');
  server = createServer(async (request, response) => {
    try {
      const url = new URL(request.url || '/', 'http://127.0.0.1');
      const filePath = resolveSafePath(rootDirectory, url.pathname);
      if (!filePath) {
        response.statusCode = 403;
        response.end('forbidden');
        return;
      }
      const file = await readFile(filePath);
      response.statusCode = 200;
      response.setHeader('Content-Type', getContentType(filePath));
      response.end(file);
    } catch (_error) {
      response.statusCode = 404;
      response.end('not found');
    }
  });

  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, '127.0.0.1', () => {
      server.off('error', reject);
      resolve();
    });
  });
}

export async function closeRemoteServer() {
  if (!server) {
    return;
  }
  const runningServer = server;
  server = undefined;
  await new Promise((resolve, reject) => {
    runningServer.close((error) => {
      if (error) {
        reject(error);
        return;
      }
      resolve();
    });
  });
}
