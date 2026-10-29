import css from './style.less';

it('applies additionalData and Less compiler options', () => {
  expect(css).toContain('color: green;');
  expect(css).toContain('width: 6px;');
  expect(css).toContain('height: 5px;');
});
