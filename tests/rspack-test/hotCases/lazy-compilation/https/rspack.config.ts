import { defineConfig } from '@rspack/cli';

import fs from 'node:fs';
import path from 'node:path';

export default defineConfig({
  lazyCompilation: {
    entries: false,
    backend: {
      server: {
        key: fs.readFileSync(path.join(import.meta.dirname, 'key.pem')),
        cert: fs.readFileSync(path.join(import.meta.dirname, 'cert.pem')),
      },
    },
  },
});
