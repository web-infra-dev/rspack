const value = require('./value');

it('should cache a consecutive loader chain until its input changes', () => {
  if (+WATCH_STEP < 3) {
    expect(value).toEqual({
      value: 'initial',
      leftRuns: +WATCH_STEP + 1,
      markedRuns: 1,
      rightRuns: 1,
      sourceMap: true,
      additionalData: true,
    });
  } else if (+WATCH_STEP === 3) {
    expect(value).toEqual({
      value: 'initial',
      leftRuns: 4,
      markedRuns: 2,
      rightRuns: 2,
      sourceMap: true,
      additionalData: true,
    });
  } else {
    expect(value).toEqual({
      value: 'changed',
      leftRuns: 5,
      markedRuns: 3,
      rightRuns: 3,
      sourceMap: true,
      additionalData: true,
    });
  }
});
