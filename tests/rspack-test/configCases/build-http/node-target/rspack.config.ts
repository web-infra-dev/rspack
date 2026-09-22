import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import customHttpClient from './custom-http-client.ts';

const tempDir = path.join(os.tmpdir(), 'rspack-build-http-node-target');

fs.mkdirSync(tempDir, { recursive: true });

export default defineConfig({
  mode: 'development',
  target: 'node',
  experiments: {
    buildHttp: {
      allowedUris: ['https://test.rspack.rs/'],
      cacheLocation: false,
      lockfileLocation: path.join(tempDir, `lock-${process.pid}.json`),
      httpClient: customHttpClient,
    },
  },
});
