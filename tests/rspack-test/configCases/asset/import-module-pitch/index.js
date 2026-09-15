import url from '!!./parent-loader!./target.txt';
import fs from 'node:fs';
import path from 'node:path';

it('should evaluate the pitch module and emit the original asset content', () => {
  expect(url).toBe('target.txt');
  expect(fs.readFileSync(path.join(__dirname, url), 'utf8')).toBe(
    'original asset content\n'.repeat(100),
  );
});
