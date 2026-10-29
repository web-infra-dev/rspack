import css from './style.less';

it('returns CSS with a composed source map', () => {
  expect(css).toContain('color: red;');
  expect(css).not.toContain('sourceMappingURL');
});
