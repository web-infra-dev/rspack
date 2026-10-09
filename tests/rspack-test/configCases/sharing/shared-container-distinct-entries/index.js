it('keeps same-identity shared container requests distinct in one compiler', async () => {
  const a = __non_webpack_require__('./A.js');
  const b = __non_webpack_require__('./B.js');
  expect((await a.get())()).toBe('a');
  expect((await b.get())()).toBe('b');
});
