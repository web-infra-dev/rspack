import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import loaderOutput from './style.css?loader';
import './style.css';

it('recognizes scroll navigation pseudo-classes in the loader and minimizer', () => {
  const minimized = readFileSync(join(__dirname, 'bundle0.css'), 'utf8');
  const rule = 'a:target-before,a:target-after{color:green}';
  expect(loaderOutput).toContain(rule);
  expect(minimized).toContain(rule);
});
