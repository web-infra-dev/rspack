it('tracks resolved imports, optional missing files and package manifests', () => {
 const css = require('./style.less');
 expect(css).toContain(`color: ${WATCH_STEP === '0' ? 'red' : 'blue'};`);
 expect(css).toContain(`width: ${WATCH_STEP === '2' ? 2 : 1}px;`);
 expect(css.includes('height: 5px;')).toBe(WATCH_STEP === '2');
});
