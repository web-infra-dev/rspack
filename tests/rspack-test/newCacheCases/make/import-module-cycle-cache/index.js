import result from './loader.mjs!./a.json';

it('should cache a caught cycle and rebuild when a dependency breaks it', async () => {
  if (COMPILER_INDEX < 2) {
    expect(result).toContain('circular build dependency');
    await NEXT_START();
  } else {
    expect(result).toBe('source: terminal');
  }
});
