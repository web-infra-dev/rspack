import { define } from 'rstack';

define.app({
  source: {
    entry: {
      'basic-react': './cases/basic-react/index.js',
    },
  },
  html: {
    template({ entryName }) {
      return `./cases/${entryName}/index.html`;
    },
  },
  server: {
    port: 8900,
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
});
