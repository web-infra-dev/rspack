// @ts-check
import fs from 'node:fs';
import {join} from 'node:path';
function replaceFileContent(filePath, replaceFn) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const newContent = replaceFn(content);
  if (newContent !== content) {
    fs.writeFileSync(filePath, newContent);
  }
}
/** @type {import('prebundle').Config} */
export default {
  dependencies: [{
    name: 'webpack-dev-server',
    ignoreDts: true,
    // this is a trick to avoid ncc compiling the dynamic import syntax
    // https://github.com/vercel/ncc/issues/935
    beforeBundle(task) {
      replaceFileContent(
          join(task.depPath, 'lib/Server.js'),
          (content) => content.replaceAll('await import', 'await __import'));
    },
    afterBundle(task) {
      replaceFileContent(
          join(task.distPath, 'index.js'),
          (content) =>
              `${content.replaceAll('await __import', 'await import')}`,
      );
    },
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
