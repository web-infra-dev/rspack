import fs from 'node:fs';
import path from 'node:path';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: 'should return false for an ignored subpath inside a PnP package',
  options() {
    return { resolve: { pnp: true, aliasFields: ['browser'] } };
  },
  build(context, compiler) {
    const directory = context.getDist();
    // The existing PnP manifest maps the rspack-pnp-test workspace to its own directory.
    fs.copyFileSync(
      path.join(import.meta.dirname, '../configCases/resolve/pnp-enable/.pnp.cjs'),
      path.join(directory, '.pnp.cjs'),
    );
    fs.writeFileSync(
      path.join(directory, 'package.json'),
      JSON.stringify({ name: 'rspack-pnp-test', browser: { './ignored.js': false } }),
    );
    fs.writeFileSync(path.join(directory, 'ignored.js'), '');
    fs.writeFileSync(path.join(directory, 'normal.js'), '');

    const resolver = compiler.resolverFactory.get('normal');
    const resolve = (request) => resolver.resolveSync({}, directory, request);
    expect(resolve('rspack-pnp-test/normal.js')).toBe(path.join(directory, 'normal.js'));
    expect(() => resolve('rspack-pnp-test/missing.js')).toThrow();
    expect(resolve('rspack-pnp-test/ignored.js')).toBe(false);
  },
};
