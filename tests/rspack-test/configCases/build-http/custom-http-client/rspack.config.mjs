import path from 'node:path';
import customHttpClient from './custom-http-client.mjs';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  entry: './index.js',
  experiments: {
    buildHttp: {
      // Test both string and regex patterns for allowedUris
      allowedUris: [
        // Allow a specific path with a string (should allow allowed-module.js)
        'http://test.rspack.rs/allowed',

        // Allow paths matching a regex pattern (should match regex-module.js)
        /^http:\/\/test\.rspack\.rs\/regex.*/,

        // Intentionally not including restricted-module.js to test blocking behavior
      ],
      cacheLocation: path.join(import.meta.dirname, 'rspack-http-cache'),
      lockfileLocation: path.join(
        import.meta.dirname,
        'rspack-http-lockfile.json',
      ),
      httpClient: customHttpClient,
    },
    css: false,
  },
};
