import { Counter } from './component.jsx';

it('should keep source map mappings inside functions compiled by React Compiler', async () => {
  const fs = require('fs');
  const { SourceMapConsumer } = require('source-map');

  expect(typeof Counter).toBe('function');

  const generated = fs.readFileSync(__filename, 'utf-8');
  // Make sure the compiler rewrote the component, so the body is generated code.
  expect(generated).toContain('react/compiler-runtime');

  const map = JSON.parse(fs.readFileSync(__filename + '.map', 'utf-8'));
  const consumer = await new SourceMapConsumer(map);
  const marker = '"react-compiler-body-marker"';
  const original = consumer.originalPositionFor(positionFor(generated, marker));
  expect(original.source).toMatch(/component\.jsx$/);
  // The marker is on line 6 of component.jsx, inside the compiled component body.
  expect(original.line).toBe(6);
});

const positionFor = (content, text) => {
  const lines = content.split(/\r?\n/);
  for (let i = 0; i < lines.length; i++) {
    const column = lines[i].indexOf(text);
    if (column >= 0) return { line: i + 1, column };
  }
  throw new Error(`Cannot find ${text}`);
};
