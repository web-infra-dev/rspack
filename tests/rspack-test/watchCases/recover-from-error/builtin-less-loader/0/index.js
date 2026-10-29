it('recovers after a missing Less import is created and tracks subsequent edits', () => {
  if (WATCH_STEP === '0') {
    expect(() => require('./style.less')).toThrow();
  } else {
    expect(require('./style.less')).toContain(`color: ${WATCH_STEP === '1' ? 'red' : 'blue'};`);
  }
});
