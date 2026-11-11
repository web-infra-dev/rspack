import result from './loader.mjs!./a.json';

it('should cache a caught cycle and rebuild when a dependency breaks it', async () => {
  if (COMPILER_INDEX < 2) {
    expect(result).toContain('circular build dependency');
  } else {
    expect(result).toBe('source: terminal');
  }
  if (COMPILER_INDEX === 0) await NEXT_START();
  if (COMPILER_INDEX === 1) await NEXT_START();
});
