const pages = require.context("./pages", true, /\.js$/);

it("should reuse a context containing dangling and looping links", async () => {
  expect(pages.keys()).toEqual(["./a.js"]);
  expect(pages("./a.js")).toBe("a");
  if (COMPILER_INDEX < 2) {
    const stats = await NEXT_START(false);
    expect(stats.modules.filter(module => module.built)).toHaveLength(0);
  }
});
