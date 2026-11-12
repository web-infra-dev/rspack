import names from "./value";

it("should rebuild a module whose context directory was removed after the build read it", async () => {
  if (COMPILER_INDEX === 0) {
    expect(names).toEqual(["a.txt"]);
    await NEXT_START(false);
  }
  if (COMPILER_INDEX === 1) {
    expect(names).toEqual([]);
    const stats = await NEXT_START(false);
    expect(stats.modules.filter(module => module.built)).toHaveLength(0);
  }
});
