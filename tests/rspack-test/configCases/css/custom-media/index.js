import './style.css';
import './disabled.css';

const fs = require('fs');
const path = require('path');

const readBundle = () =>
  fs.readFileSync(path.join(__dirname, 'bundle0.css'), 'utf-8');

it('should resolve custom media like webpack', () => {
  const css = readBundle();

  expect(css).toContain('@media (max-width: 30em)');
  expect(css).toContain(
    '@media ((color) or (hover)) and (min-width: 100px)',
  );
  expect(css).toContain('@media (not (hover: hover))');
  expect(css).toContain('@media (min-width: 60em)');
  expect(css).toContain('@media (max-width: 30em), (min-width: 60em)');
  expect(css).toMatch(/@media screen\s*\{/);
  expect(css).toContain('@media screen and (min-width: 100px)');
  expect(css).toMatch(/@media not all\s*\{\s*\.legacy/);
  expect(css).toMatch(/@media all\s*\{\s*\.always/);
  expect(css).toMatch(/@media not all\s*\{\s*\.never/);
  expect(css).toMatch(/@media \(min-width: 100px\)\s*\{\s*\.and-true/);
  expect(css).toMatch(/@media not all\s*\{\s*\.and-false/);
  expect(css).toMatch(/@media \(min-width: 100px\)\s*\{\s*\.or-false/);
  expect(css).toMatch(/@media not all\s*\{\s*\.type-and-false/);
  expect(css).toMatch(/@media all\s*\{\s*\.not-false/);
  expect(css).toMatch(/@media \(max-width: 30em\)\s*\{\s*\.nested/);
  expect(css).toMatch(
    /@media \(min-width: 100px\)\s*\{\s*\.nested-boolean/,
  );
  expect(css).toContain('((max-width: 30em) or (min-width: 80em))');
  expect(css).toContain('@media (min-width: 1px) and (hover)');
  expect(css).toContain('calc(1px + 1px)');
  expect(css).toContain(
    '@media not ((min-width: 1px) and (max-width: 2px))',
  );
  expect(css).toContain(
    '@media ((min-width: 1px) and (max-width: 2px)) or (hover)',
  );
  expect(css).toContain(
    '@media ((min-width: 1px) or (hover)) and (color)',
  );
  expect(css).toContain(
    '@media ((color) or (not ((min-width: 1px) and (max-width: 2px))))',
  );
  expect(css).toContain(
    '@media (min-width: 1px) and (max-width: 2px) and (hover)',
  );
  expect(css).toMatch(/@media all\s*\{\s*\.always-or/);
  expect(css).toContain('@media screen and (hover)');
  expect(css).toContain('@media print, (max-width: 30em)');
  expect(css).toMatch(/@media not all\s*\{\s*\.never-and/);

  expect(css).not.toContain('@custom-media --narrow');
  expect(css).not.toContain('@custom-media --wide');
});

it('should honor module.parser.css.customMedia = false', () => {
  const css = readBundle();

  expect(css).toContain('@custom-media --disabled (max-width: 10em)');
  expect(css).toContain('@media (--disabled)');
});
