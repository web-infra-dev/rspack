import css from './style.less';
it('resolves aliases, exports, tilde imports and nested Less scopes', () => {
  for (const declaration of ['color: purple;', 'width: 12px;', 'height: 7px;', 'padding: 9px;', 'margin: 3px;', 'top: 4px;', 'left: 5px;', 'right: 6px;', 'bottom: 8px;', 'z-index: 2;']) expect(css).toContain(declaration);
  expect(css).toContain('@import "external.css"');
  expect(css).not.toContain('wrong');
});
