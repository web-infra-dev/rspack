import value from './convert!./input.txt';

it('should reuse loader resolutions and select newly added loaders', async () => {
  if (COMPILER_INDEX === 0) {
    expect(value).toBe('js:input');
    await NEXT_HMR();
    expect(value).toBe('js:input');
    await NEXT_START();
  } else if (COMPILER_INDEX === 1) {
    expect(value).toBe('cjs:input');
    await NEXT_START();
  } else {
    expect(value).toBe('js:input');
    if (COMPILER_INDEX === 2) await NEXT_START();
  }
});
