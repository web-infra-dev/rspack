it('should apply sharing when the previous build has no shared config', async () => {
  const { double } = await import('./app');
  expect(double(2)).toBe(4);

  if (COMPILER_INDEX === 0) {
    await NEXT_START();
  }

  if (COMPILER_INDEX === 1) {
    expect(
      __STATS__.modules.some((module) =>
        module.name.startsWith('consume shared module (default) ./shared'),
      ),
    ).toBe(true);
  }
});
