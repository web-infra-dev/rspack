it("afterOptimizeChunkIds should be called with chunks that already have ids", () => {
  expect(1).toBe(1);
});

it("should load the async chunk", () => {
  return import("./async").then(({ default: value }) => {
    expect(value).toBe("async");
  });
});
