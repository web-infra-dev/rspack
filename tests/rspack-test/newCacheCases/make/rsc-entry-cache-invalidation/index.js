import fs from 'node:fs';
import path from 'node:path';

it('should emit the current client CSS in import order after restart', async () => {
  const css = fs.readFileSync(path.join(__dirname, 'clients.css'), 'utf8');
  const colors = css.match(/red|blue|green/g);
  if (COMPILER_INDEX === 0) {
    expect(colors).toEqual(['red', 'blue']);
    await NEXT_START();
  } else {
    expect(colors).toEqual(['blue', 'green']);
  }
});
