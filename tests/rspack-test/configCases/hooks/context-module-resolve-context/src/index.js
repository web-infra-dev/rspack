it("should preserve the resolve context through context module factory hooks", () => {
  const name = "value";
  expect(require(`./local/${name}`).default).toBe(42);
});

it("should relocate import.meta.glob when a hook changes the resolve context", () => {
  const modules = import.meta.glob("./local/*.js", { eager: true });
  expect(Object.keys(modules)).toEqual(["./local/value.js"]);
  expect(modules["./local/value.js"].default).toBe(42);
});

it("should keep the glob context stable when afterResolve changes context", () => {
  const modules = import.meta.glob("./after-source/*.js", { eager: true });
  expect(Object.keys(modules)).toEqual(["./after-source/value.js"]);
  expect(modules["./after-source/value.js"].default).toBe(44);
});
