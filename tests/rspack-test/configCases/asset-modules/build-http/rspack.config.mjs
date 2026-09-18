import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  experiments: {
    buildHttp: {
      allowedUris: [() => true],
      lockfileLocation: path.resolve(
        import.meta.dirname,
        './lock-files/lock.json',
      ),
      cacheLocation: path.resolve(import.meta.dirname, './lock-files/test'),
    },
  },
};
