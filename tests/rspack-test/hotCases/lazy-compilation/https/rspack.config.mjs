import fs from 'node:fs';
import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  lazyCompilation: {
    entries: false,
    backend: {
      server: {
        key: fs.readFileSync(path.join(import.meta.dirname, 'key.pem')),
        cert: fs.readFileSync(path.join(import.meta.dirname, 'cert.pem')),
      },
    },
  },
};
