it('allows disabling source maps independently of devtool', () => {
  expect(require('./style.less')).toContain('color: red;');
});
