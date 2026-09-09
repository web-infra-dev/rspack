import value from './loader!./file';

it('should snapshot context dependency timestamp changes', async () => {
  if (COMPILER_INDEX === 0) {
    expect(value).toBe(1);
    await NEXT_START();
  }
  if (COMPILER_INDEX === 1) {
    // A changed context timestamp must invalidate the cached module.
    expect(value).toBe(2);
    await NEXT_START();
  }
  if (COMPILER_INDEX === 2) {
    // An unchanged context must allow cache reuse despite file.js changing again.
    expect(value).toBe(2);
  }
});

module.hot.accept('./file');
