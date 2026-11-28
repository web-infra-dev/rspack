import * as rootPlain from 'root-plain';
import * as rootEsm from 'root-esm';
import * as rootDefault from 'root-default';
import * as commonjsPlain from 'commonjs-plain';
import * as commonjsEsm from 'commonjs-esm';
import * as commonjsDefault from 'commonjs-default';
import * as commonjs2Plain from 'commonjs2-plain';
import * as commonjs2Esm from 'commonjs2-esm';
import * as commonjs2Default from 'commonjs2-default';
import * as amdPlain from 'amd-plain';
import * as amdEsm from 'amd-esm';
import * as amdDefault from 'amd-default';
import * as allPlain from 'all-plain';
import * as allEsm from 'all-esm';
import * as allDefault from 'all-default';

for (const [branch, plain, esm, cjs] of [
  ['root', rootPlain, rootEsm, rootDefault],
  ['commonjs', commonjsPlain, commonjsEsm, commonjsDefault],
  ['commonjs2', commonjs2Plain, commonjs2Esm, commonjs2Default],
  ['amd', amdPlain, amdEsm, amdDefault],
  ['all', allPlain, allEsm, allDefault],
]) {
  it(`should ignore interop when the ${branch} UMD target selects a property`, () => {
    const selected = branch === 'commonjs2' || branch === 'all';
    const value = selected ? 'nested default' : 'the default';
    expect(plain.default).toEqual(STRICT_IMPORTER ? expect.objectContaining({ default: value }) : value);
    expect(esm.default).toBe(plain.default);
    expect(cjs.default).toBe(plain.default);
    expect(esm.named).toBe(selected ? 43 : 42);
    expect(cjs.named).toBe(plain.named);
  });
}
