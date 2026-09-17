import path from 'node:path';

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        exclude: [/index\.js/],
        use: './loader0.js',
      },
      {
        exclude: [/index\.js/],
        use: './loader1.js',
        issuer: {
          not: [/index\.js/],
        },
      },
      {
        exclude: [/index\.js/],
        use: './loader2.js',
        issuer: {
          and: [/1\.js/, path.resolve(import.meta.dirname, 'lib')],
        },
      },
      {
        exclude: [/index\.js/],
        use: './loader3.js',
        issuer: {
          or: [/1\.js/, /2\.js/],
        },
      },
    ],
  },
};
