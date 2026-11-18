import './style.css';
import fs from 'node:fs';
import path from 'node:path';

it('should reuse unchanged extracted CSS and rebuild changed CSS across restarts', async () => {
  const css = fs.readFileSync(path.join(__dirname, 'main.css'), 'utf8');
  const color = COMPILER_INDEX < 2 ? 'red' : 'blue';
  expect(css.trim()).toBe(`.test { color: ${color}; }`);
  if (COMPILER_INDEX < 2) await NEXT_START();
});
