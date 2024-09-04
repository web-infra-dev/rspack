// @ts-check

/** @type {import('prebundle').Config} */
export default {
  dependencies: [{
    name: 'webpack-dev-server',
    ignoreDts: true,
    externals: {
      // the following externals are from webpack-dev-server's dependencies
      // https://github.com/webpack/webpack-dev-server/blob/master/package.json#L48
      'ansi-html-community': 'ansi-html-community',
      'bonjour-service': 'bonjour-service',
      chokidar: 'chokidar',
      colorette: 'colorette',
      compression: 'compression',
      'connect-history-api-fallback': 'connect-history-api-fallback',
      express: 'express',
      'graceful-fs': 'graceful-fs',
      'html-entities': 'html-entities',
      'http-proxy-middleware': 'http-proxy-middleware',
      'ipaddr.js': 'ipaddr.js',
      'launch-editor': 'launch-editor',
      open: 'open',
      'p-retry': 'p-retry',
      'schema-utils': 'schema-utils',
      selfsigned: 'selfsigned',
      'serve-index': 'serve-index',
      sockjs: 'sockjs',
      spdy: 'spdy',
      'webpack-dev-middleware': 'webpack-dev-middleware',
      ws: 'ws',
      webpack: 'webpack',
      'webpack-cli': 'webpack-cli'
    }
  }]
};
