import './style.css';
import './dependency.js';

it('The dependOn chunk must be loaded before the common chunk.', async () => {
  const fs = await eval(`import("fs")`);
  const path = await eval(`import("path")`);
  const source = fs.readFileSync(__filename, 'utf-8');
  const snapshotFile = /^export const __rspack_modules =/m.test(source)
    ? path.join(__SNAPSHOT__, 'runtimeModeSnapshot', 'foo.mjs.txt')
    : path.join(__SNAPSHOT__, 'foo.mjs.txt');

  const normalize = value => value.replace(/\r\n/g, '\n').trimEnd();
  expect(normalize(source)).toBe(
    normalize(fs.readFileSync(snapshotFile, 'utf-8')),
  );
});
