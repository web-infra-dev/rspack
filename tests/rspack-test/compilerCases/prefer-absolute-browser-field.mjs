import fs from 'node:fs';
import path from 'node:path';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: 'should honor browser aliases before falling back to roots',
  skip: process.platform === 'win32',
  build(context, compiler) {
    const absolute = context.getDist('absolute');
    const root = context.getDist('root');
    const mirror = path.join(root, absolute);
    fs.mkdirSync(absolute, { recursive: true });
    fs.mkdirSync(mirror, { recursive: true });
    fs.writeFileSync(
      path.join(absolute, 'package.json'),
      JSON.stringify({
        browser: { './ignored.js': false, './redirected.js': './missing.js' },
      }),
    );
    for (const name of ['ignored.js', 'redirected.js']) {
      fs.writeFileSync(path.join(absolute, name), '');
      fs.writeFileSync(path.join(mirror, name), '');
    }

    const resolver = compiler.resolverFactory.get('normal', {
      preferAbsolute: true,
      roots: [root],
      aliasFields: ['browser'],
    });
    const ignored = path.join(absolute, 'ignored.js');
    const redirected = path.join(absolute, 'redirected.js');
    const rootsFirst = resolver.withOptions({ preferAbsolute: false });
    expect(resolver.resolveSync({}, absolute, ignored)).toBe(false);
    expect(resolver.resolveSync({}, absolute, redirected)).toBe(
      path.join(mirror, 'redirected.js'),
    );
    expect(rootsFirst.resolveSync({}, absolute, ignored)).toBe(
      path.join(mirror, 'ignored.js'),
    );
  },
};
