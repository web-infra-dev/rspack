import css from './style.less';

it('resolves a pnpm package dependency from the real imported filename', () => {
  expect(css).toBe('.from-theme {\n  color: purple;\n}\n.box {\n  color: purple;\n}\n');
});
