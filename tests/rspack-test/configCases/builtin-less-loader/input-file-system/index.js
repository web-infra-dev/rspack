it('reads imported Less through the configured input file system', () => {
  expect(require('./style.less')).toContain('color: purple;');
});
