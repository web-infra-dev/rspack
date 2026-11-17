it('should restore a module with literal portable markers in its layer', async () => {
  if (COMPILER_INDEX === 0) {
    const stats = await NEXT_START(false);
    expect(stats.modules.filter(module => module.built)).toHaveLength(0);
  }
});
