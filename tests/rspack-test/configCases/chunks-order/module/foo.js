import './style.css';
import './dependency.js';

it('The dependOn chunk must be loaded before the common chunk.', async () => {
  const fs = await eval(`import("fs")`);
  const path = await eval(`import("path")`);
  const source = fs.readFileSync(__filename, 'utf-8');

  expect(source).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'foo.mjs.txt'));
});
