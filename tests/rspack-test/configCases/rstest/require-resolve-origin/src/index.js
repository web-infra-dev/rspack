const name = './target';

const a = require.resolve('./target');
const b = require.resolve(name);
const c = require.resolve('./target', { paths: [__dirname] });
const d = require.resolve(require('./nested').name);
const e = require.resolve(/* webpackIgnore: true */ './ignored');

function shadowed(require) {
  return require.resolve('./shadowed');
}

console.log(a, b, c, d, e, shadowed);
