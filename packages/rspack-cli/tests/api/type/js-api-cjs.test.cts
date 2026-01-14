const rspackCjsDefaultRequire = require('@rspack/core');
const { rspack: rspackCjsNamedRequire } = require('@rspack/core');

import assert from 'node:assert';

type IsFunction<T> = T extends (...args: any[]) => any ? true : false;

// not real Node.js behavior
// https://github.com/web-infra-dev/rspack/issues/8095
describe.concurrent(
  'js-api-type should be correct when requiring from @rspack/core',
  () => {
    it('cjs default require', async () => {
      // const rspack = require('@rspack/core')
      type Truthy = IsFunction<typeof rspackCjsDefaultRequire>;
      const truthy: Truthy = true;
      truthy;
      assert(rspackCjsDefaultRequire.BannerPlugin);
      assert(typeof rspackCjsDefaultRequire === 'object');
      const compiler = rspackCjsDefaultRequire.default({});
      assert(compiler);
    });

    it('cjs named require', async () => {
      // const { rspack } = require('@rspack/core')
      type Truthy = IsFunction<typeof rspackCjsNamedRequire>;
      const truthy: Truthy = true;
      truthy;
      assert(rspackCjsNamedRequire.BannerPlugin);
      assert(typeof rspackCjsNamedRequire === 'function');
      const compiler = rspackCjsNamedRequire({});
      assert(compiler);
    });

    it('rspack.default', async () => {
      // const { rspack } = require('@rspack/core')
      // rspack.default should be undefined
      assert(!(rspackCjsNamedRequire as any).default);
      // const rspack = require('@rspack/core')
      // rspack.default should be defined
      assert(!!(rspackCjsDefaultRequire as any).default);
    });
  },
);
