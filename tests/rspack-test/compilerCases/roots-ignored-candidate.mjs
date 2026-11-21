import fs from 'node:fs';
import path from 'node:path';

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: 'should stop trying roots when a candidate is ignored',
  build(context, compiler) {
    const first = context.getDist('first');
    const second = context.getDist('second');
    fs.mkdirSync(first, { recursive: true });
    fs.mkdirSync(second, { recursive: true });
    fs.writeFileSync(
      path.join(first, 'package.json'),
      JSON.stringify({ browser: { './ignored.js': false } }),
    );
    fs.writeFileSync(path.join(first, 'ignored.js'), '');
    fs.writeFileSync(path.join(second, 'ignored.js'), '');
    fs.writeFileSync(path.join(second, 'fallback.js'), '');

    const resolver = compiler.resolverFactory.get('normal', {
      roots: [first, second],
      aliasFields: ['browser'],
    });
    const resolve = (request) =>
      resolver.resolveSync({}, context.getDist(), request);
    expect(resolve('/fallback.js')).toBe(path.join(second, 'fallback.js'));
    expect(resolve('/ignored.js')).toBe(false);
  },
};
