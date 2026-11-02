import './style.css';

const fs = require('fs');
const path = require('path');

it('should leave unresolvable custom media references untouched and warn', () => {
  const css = fs.readFileSync(path.join(__dirname, 'bundle0.css'), 'utf-8');

  expect(css).toContain('(--undefined-mq)');
  expect(css).toContain('(--defined: 1px)');
  expect(css).toContain('(--loop)');
  expect(css).toContain('(min-width: 1px) and (--tvish)');
  expect(css).toContain('(--parenthesised-type)');
  expect(css).toContain('(--compound)');
  expect(css).toContain('(--bad-inner)');
  expect(css).toContain('(--bad-segment)');
  expect(css).toContain('(--type-segment)');
  expect(css).not.toContain('@custom-media');
});
