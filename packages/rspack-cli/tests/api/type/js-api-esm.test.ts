import assert from 'node:assert';
import rspackEsmDefaultImport, {
  rspack as rspackEsmNamedImport,
} from '@rspack/core';

type IsFunction<T> = T extends (...args: any[]) => any ? true : false;

// not real Node.js behavior
// https://github.com/web-infra-dev/rspack/issues/8095
describe.concurrent(
  'js-api-type should be correct when importing from @rspack/core',
  () => {
    it('esm default import', async () => {
      assert(rspackEsmDefaultImport);
      assert(typeof rspackEsmDefaultImport === 'function');
      assert(rspackEsmDefaultImport.BannerPlugin);
    });

    it('esm named import', async () => {
      type Truthy = IsFunction<typeof rspackEsmNamedImport>;
      const truthy: Truthy = true;
      truthy;
      assert(rspackEsmNamedImport.BannerPlugin);
      assert(typeof rspackEsmNamedImport === 'function');
      const compiler = rspackEsmNamedImport({});
      assert(compiler);
    });

    it('rspack.default should not exist in esm import', async () => {
      assert(!(rspackEsmNamedImport as any).default);
      assert(!(rspackEsmDefaultImport as any).default);
    });
  },
);
