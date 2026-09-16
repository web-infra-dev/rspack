/** @type {import('@rspack/core').Configuration} */
module.exports = {
  target: 'web',
  experiments: {
    buildHttp: {
      frozen: false,
      allowedUris: [
        'http://allowed.example',
        'http://path.example/modules/',
        'https://',
      ],
      cacheLocation: false,
      httpClient: async (url) => {
        if (url === 'http://allowed.example/redirect') {
          return {
            status: 302,
            headers: {
              location: 'http://allowed.example@blocked.example/redirected.js',
              'cache-control': 'no-store',
            },
            body: Buffer.from(''),
          };
        }

        throw new Error(`Disallowed URL reached the HTTP client: ${url}`);
      },
    },
  },
};
