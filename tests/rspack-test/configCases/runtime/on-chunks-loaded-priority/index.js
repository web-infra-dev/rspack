it("runs a ready even-priority callback while an earlier chunk is pending", () => {
  const onChunksLoaded = __webpack_require__.O;
  const loaded = new Set(["ready"]);
  const calls = [];
  onChunksLoaded.test = id => loaded.has(id);

  onChunksLoaded(undefined, ["pending"], () => calls.push(1), 1);
  onChunksLoaded(undefined, ["ready"], () => calls.push(2), 2);
  onChunksLoaded(undefined, ["ready"], () => calls.push(3), 3);
  onChunksLoaded();
  expect(calls).toEqual([2]);

  loaded.add("pending");
  onChunksLoaded();
  expect(calls).toEqual([2, 1, 3]);
});
