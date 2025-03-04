__webpack_require__("./foo.js");

it("should addInclude foo.js", () => {
  expect(STATE.foo).toBe(42);
  const builtModules = Object.fromEntries(__STATS__.modules.map(m => [m.name, m.built]));
  expect(builtModules).toEqual({ "./index.js": true, "./foo.js": true });
});
