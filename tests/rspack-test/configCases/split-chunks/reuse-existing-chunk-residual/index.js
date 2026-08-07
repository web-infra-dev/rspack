it("should preserve a reused destination at lower priorities", async () => {
  const modules = await Promise.all([
    import(/* webpackChunkName: "Foo" */ "./Foo"),
    import(/* webpackChunkName: "Bar" */ "./Bar"),
    import(/* webpackChunkName: "ReusableUtil" */ "./util"),
  ]);

  expect(modules.map((module) => module.default)).toEqual([
    "Foo:util",
    "Bar:util",
    "util",
  ]);
});
