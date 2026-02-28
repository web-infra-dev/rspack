import assert from 'node:assert';
import {
  rspackEsmDefaultImport,
  rspackEsmNamedImport,
  rspackNamespaceImport,
} from './api-wrapper.mjs';

// https://github.com/web-infra-dev/rspack/issues/8095
describe('js-api-type should be correct when importing from @rspack/core', () => {
  it('esm default import', async () => {
    // import rspack from '@rspack/core'
    assert(typeof rspackEsmDefaultImport === 'function');
    assert(rspackEsmDefaultImport.BannerPlugin);
    const compiler = rspackEsmDefaultImport({});
    assert(compiler);
  });

  it('esm namespace import', async () => {
    // import * as rspack from '@rspack/core'
    assert(typeof rspackNamespaceImport === 'object');
    assert(typeof rspackNamespaceImport.rspack === 'function');
    assert(rspackNamespaceImport.BannerPlugin);
    const compiler1 = rspackNamespaceImport.rspack({});
    assert(compiler1);
    const compiler2 = rspackNamespaceImport.default({});
    assert(compiler2);
  });

  it('esm named import', async () => {
    // import { rspack } from '@rspack/core'
    assert(typeof rspackEsmNamedImport === 'function');
    assert(rspackEsmNamedImport.BannerPlugin);
    const compiler = rspackEsmNamedImport({});
    assert(compiler);
  });

  it('rspack.default should not exist in esm named and default import', async () => {
    assert(!(rspackEsmNamedImport as any).default);
    assert(!(rspackEsmDefaultImport as any).default);
  });

  it('rspack.default should exist in esm namespace import', async () => {
    assert((rspackNamespaceImport as any).default);
    assert(typeof (rspackNamespaceImport as any).default === 'function');
    assert(rspackNamespaceImport.default === rspackNamespaceImport.rspack);
  });
});
