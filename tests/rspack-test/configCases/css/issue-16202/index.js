import { control, important, nameImportant } from './index.module.css';

const fs = require('fs');
const path = require('path');

it('renames CSS module keyframes before !important', () => {
  expect(control).toBe('control-scoped');
  expect(important).toBe('important-scoped');
  expect(nameImportant).toBe('nameImportant-scoped');

  const css = fs.readFileSync(path.resolve(__dirname, 'bundle0.css'), 'utf-8');
  const output = css.replace(/\s+/g, ' ');
  expect(output).toContain('@keyframes spin-scoped');
  expect(output).toContain('animation: spin-scoped 1s linear infinite;');
  expect(output).toContain('animation: spin-scoped 1s linear infinite !important;');
  expect(output).toContain('animation-name: spin-scoped !important;');
});
