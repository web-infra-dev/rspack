it('composes incoming maps and supports sourceMap without devtool', () => {
  expect(require('./style.less')).toContain('color: red;');
});
