import './style.less';
import fs from 'fs';
import path from 'path';

it('feeds compiled CSS to the builtin CSS pipeline', () => {
  const css = fs.readFileSync(path.join(__dirname, 'style.css'), 'utf8');
  expect(css).toMatch(/\.box\s*\{\s*color:\s*red/);
});
