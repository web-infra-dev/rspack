import fs from 'fs';
import path from 'path';

it('json keys with special characters do not produce invalid identifiers', async () => {
  const bundle = fs.readFileSync(
    path.resolve(import.meta.dirname, './bundle.lib.mjs'),
    'utf-8',
  );

  // Should never emit an unquoted special character in an identifier position
  // (e.g. `var _!top = ...` or `var with space = ...`).
  expect(bundle).not.toMatch(/^var\s+(?:\S+\s+\S+|\S*[!.]\S*)\s*=/m);

  // The named exports for non-identifier keys must be string-quoted aliases.
  expect(bundle).toContain('as "!top"');
  expect(bundle).toContain('as "with space"');
  expect(bundle).toContain('as "a.b"');

  const mod = await import(/*webpackIgnore: true*/'./bundle.lib.mjs');
  expect(mod['!top']).toEqual([1]);
  expect(mod['with space']).toBe('x');
  expect(mod['a.b']).toBe(2);
  expect(mod.normal).toBe(3);
  expect(mod.default).toEqual({
    '!top': [1],
    'with space': 'x',
    'a.b': 2,
    normal: 3,
  });
});
