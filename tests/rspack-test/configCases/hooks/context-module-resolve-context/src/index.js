it("should preserve the resolve context through context module factory hooks", () => {
  const name = "value";
  expect(require(`./local/${name}`).default).toBe(42);
});

it("should relocate import.meta.webpackContext when beforeResolve changes context", () => {
  const context = import.meta.webpackContext("./local", {
    recursive: false,
    regExp: /value\.js$/,
  });
  expect(context("./value.js").default).toBe(42);
});

it("should keep the source glob base when a hook changes the resolve context", () => {
  const modules = import.meta.glob("./local/*.js", { eager: true });
  expect(Object.keys(modules)).toEqual([]);
});

it("should use an explicit glob request and recursive value returned by beforeResolve", () => {
  const modules = import.meta.glob("./request-override/*.js", { eager: true });
  expect(Object.keys(modules)).toEqual(["./request-override/value.js"]);
  expect(modules["./request-override/value.js"].default).toBe(45);
});

it("should not recompile mixed glob patterns after beforeResolve changes context", () => {
  const modules = import.meta.glob(["../shared/*.js", "/shared/*.js"], {
    eager: true,
  });
  expect(Object.keys(modules)).toEqual([]);
});

it("should keep the webpackContext stable when afterResolve changes context", () => {
  const context = import.meta.webpackContext("./after-source", {
    recursive: false,
    regExp: /value\.js$/,
  });
  expect(context("./value.js").default).toBe(44);
});

it("should keep the glob context stable when afterResolve changes context", () => {
  const modules = import.meta.glob("./after-source/*.js", { eager: true });
  expect(Object.keys(modules)).toEqual(["./after-source/value.js"]);
  expect(modules["./after-source/value.js"].default).toBe(44);
});

it("should use an afterResolve resource override without changing glob coordinates", () => {
  const modules = import.meta.glob("/**/after-resource/*.js", {
    eager: true,
    query: "?after-resource-override",
  });
  expect(Object.keys(modules)).toEqual([
    "/fixtures/after-resource/value.js",
  ]);
  expect(modules["/fixtures/after-resource/value.js"].default).toBe(48);
});
