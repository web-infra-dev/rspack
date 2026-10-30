import css from './style.less';
it('resolves tilde package exports and retains import metadata', () => {
  expect(css).toContain('color: purple;');
  expect(css).toContain('width: 11px;');
  expect(css).toContain('height: 7px;');
  expect(css).toContain('node_modules/exported/images/icon.svg');
  expect(css).not.toContain('.unused');
});
