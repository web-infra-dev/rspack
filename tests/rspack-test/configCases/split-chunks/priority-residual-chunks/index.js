it("should split lower-priority residual chunks", async () => {
  const modules = await Promise.all([
    import(/* webpackChunkName: "Foo" */ "./Foo"),
    import(/* webpackChunkName: "Bar" */ "./Bar"),
    import(/* webpackChunkName: "Other" */ "./Other"),
    import(/* webpackChunkName: "Other2" */ "./Other?1"),
    import(/* webpackChunkName: "Extra" */ "./Extra"),
  ]);

  expect(modules.map((module) => module.default)).toEqual([
    "Foo:util",
    "Bar:util",
    "Other:util:helper",
    "Other:util:helper",
    "Extra:helper",
  ]);
});
