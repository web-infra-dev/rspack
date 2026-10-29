it('rebuilds for imported and newly created optional Less files', () => {
  const css = require('./style.less');
  expect(css).toContain(`color: ${WATCH_STEP === '0' ? 'red' : 'blue'};`);
  expect(css.includes('height: 5px;')).toBe(WATCH_STEP === '2');
});
