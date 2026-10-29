import css from './style.less';

it('compiles Less variables, imports, mixins and nesting', () => {
  expect(css).toContain('.box {\n  color: red;\n  width: 12px;\n}');
  expect(css).toContain('.box .child {\n  color: blue;\n}');
  expect(css).toMatch(/@import ['"]external\.css['"];/);
  expect(css).toContain('data:image/svg+xml');
});

it('supports inline builtin loader requests', () => {
  expect(require('!!raw-loader!builtin:less-loader!./style.less').default).toBe(css);
});
