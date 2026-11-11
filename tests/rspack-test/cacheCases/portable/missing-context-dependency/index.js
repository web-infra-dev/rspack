import value from "./value";

it("should reuse a module while its optional context dependency stays absent", async () => {
  expect(value).toBe("value");
  if (COMPILER_INDEX < 2) {
    const stats = await NEXT_START(false);
    expect(stats.modules.filter(module => module.built)).toHaveLength(0);
  }
});
