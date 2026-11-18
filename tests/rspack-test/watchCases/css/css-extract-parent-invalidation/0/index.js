import './a.css';
import './b.css';
import fs from 'fs';
import path from 'path';

it('should update shared CSS when its producers change', () => {
  const css = fs.readFileSync(path.join(__dirname, 'main.css'), 'utf8');
  expect(css).toContain('.a { color: black; }');
  expect(css).toContain('.b { color: white; }');
  if (WATCH_STEP === '3') {
    expect(css).not.toContain('.shared');
  } else {
    const color = Number(WATCH_STEP) < 2 ? 'red' : 'blue';
    expect(css).toContain(`.shared { color: ${color}; }`);
    expect(css.match(/\.shared/g)).toHaveLength(1);
  }
});
