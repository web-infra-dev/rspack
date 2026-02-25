import assert from 'node:assert';
const {
  rspackCjsDefaultRequire,
  rspackCjsNamedRequire,
  webpackCjsNamedRequire,
} = require('./api-wrapper.cjs');

// https://github.com/web-infra-dev/rspack/issues/8095
describe('js-api-type should be correct when requiring from @rspack/core', () => {
  it('cjs default require', async () => {
    // const rspack = require('@rspack/core')
    assert(typeof rspackCjsDefaultRequire === 'function');
    assert(rspackCjsDefaultRequire.BannerPlugin);
    const compiler = rspackCjsDefaultRequire({});
    assert(compiler);
  });

  it('cjs named require', async () => {
    // const { rspack } = require('@rspack/core')
    assert(typeof rspackCjsNamedRequire === 'function');
    // const { webpack } = require('@rspack/core')
    assert(typeof webpackCjsNamedRequire === 'function');
    assert(rspackCjsNamedRequire.BannerPlugin);
    const compiler = rspackCjsNamedRequire({});
    assert(compiler);
  });

  it('rspack.default should not exist in cjs require', async () => {
    // const { rspack } = require('@rspack/core')
    assert(!(rspackCjsNamedRequire as any).default);
    // const rspack = require('@rspack/core')
    assert(!(rspackCjsDefaultRequire as any).default);
  });
});
