import dep from 'dep';

it('should reuse resolutions and invalidate them when the package map changes', async () => {
  if (COMPILER_INDEX === 0) {
    expect(dep).toBe('dep@1.0.0');
    await NEXT_START();
  } else if (COMPILER_INDEX === 1) {
    // The package map is unchanged, so the cached resolution is reused.
    expect(dep).toBe('dep@1.0.0');
    await NEXT_START();
  } else {
    // The package map now selects another package for the dependency.
    expect(dep).toBe('dep@2.0.0');
  }
});
