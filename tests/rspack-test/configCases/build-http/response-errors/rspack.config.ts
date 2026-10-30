import { createServer, type Server } from 'node:http';
import path from 'node:path';
import { brotliCompressSync, deflateSync, gzipSync } from 'node:zlib';
import { defineConfig, definePlugin } from '@rspack/cli';
import cases from './cases.ts';

const source = Buffer.from(
  'it("decodes the HTTP response", () => expect(42).toBe(42));',
);
const encoded = {
  plain: source,
  gzip: gzipSync(source),
  br: brotliCompressSync(source),
  deflate: deflateSync(source),
};

export default cases.map((variant, index) => {
  let url: string;
  let server: Server;
  return defineConfig({
    entry: () => url,
    experiments: {
      buildHttp: {
        allowedUris: [/^http:\/\/127\.0\.0\.1:\d+\/module\.js$/],
        frozen: false,
        cacheLocation: false,
        lockfileLocation: path.join(
          import.meta.dirname,
          'lockfiles',
          `${variant}.rspack.lock`,
        ),
      },
    },
    plugins: [
      definePlugin((compiler) => {
        compiler.hooks.beforeRun.tapPromise('ResponseServer', async () => {
          server = createServer((_request, response) => {
            response.setHeader('Content-Type', 'application/javascript');
            if (variant === 'aborted') {
              response.writeHead(200, { 'Content-Length': 1000 });
              response.write(source);
              setTimeout(() => response.destroy(), 30);
            } else if (variant === 'truncated-gzip') {
              response.setHeader('Content-Encoding', 'gzip');
              response.end(encoded.gzip.subarray(0, -4));
            } else if (variant.startsWith('invalid-')) {
              response.setHeader('Content-Encoding', variant.slice(8));
              response.end('invalid compressed module');
            } else {
              if (variant !== 'plain') {
                response.setHeader('Content-Encoding', variant);
              }
              response.end(encoded[variant as keyof typeof encoded]);
            }
          });
          await new Promise<void>((resolve, reject) => {
            server.once('error', reject);
            server.listen(0, '127.0.0.1', resolve);
          });
          server.unref();
          const address = server.address();
          if (!address || typeof address === 'string') {
            throw new Error('Expected a TCP server address');
          }
          url = `http://127.0.0.1:${address.port}/module.js`;
        });
        compiler.hooks.done.tapPromise('ResponseServer', async (stats) => {
          await new Promise<void>((resolve, reject) => {
            server.close((error) => (error ? reject(error) : resolve()));
          });
          expect(stats.hasErrors(), variant).toBe(index >= 4);
        });
        compiler.hooks.failed.tap('ResponseServer', () => {
          server?.closeAllConnections();
          server?.close();
        });
      }),
    ],
  });
});
