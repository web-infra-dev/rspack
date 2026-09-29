import { defineConfig } from '@rspack/cli';
import type { HttpUriOptions } from '@rspack/core';

type HttpClient = NonNullable<HttpUriOptions['httpClient']>;

export default defineConfig({
  target: 'web',
  experiments: {
    buildHttp: {
      frozen: false,
      allowedUris: [
        'not a URL',
        'HTTP://ALLOWED.EXAMPLE:80',
        'http://path.example/old/../modules/',
        /^http:\/\/regex\.example\/module\.js$/,
      ],
      cacheLocation: false,
      httpClient: async (url): ReturnType<HttpClient> => {
        if (url === 'http://allowed.example/redirect') {
          return {
            status: 302,
            headers: {
              location: './redirected.js',
              'cache-control': 'no-store',
            },
            body: Buffer.from(''),
          };
        }

        return {
          status: 200,
          headers: {
            'content-type': 'application/javascript',
            'cache-control': 'no-store',
          },
          body: Buffer.from(`export default ${JSON.stringify(url)};`),
        };
      },
    },
  },
});
