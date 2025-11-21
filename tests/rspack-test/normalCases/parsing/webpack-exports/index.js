it("should contain only one export from webpackExports from module", function () {
  const f = module => {
    expect(module.usedExports).toEqual(["usedExports"]);
  };
  return import(/* webpackExports: "usedExports" */ "./dir12/a?1").then(f);
});

it("should contain only webpackExports from module", function () {
  const f = module => {
    expect(module.usedExports).toEqual(["a", "b", "usedExports"]);
  };
  return import(
    /* webpackExports: ["a", "usedExports", "b"] */ "./dir12/a?2"
  ).then(f);
});

it("should contain only webpackExports from module in eager mode", function () {
  const f = module => {
    expect(module.usedExports).toEqual(["a", "b", "usedExports"]);
  };
  return import(
    /*
    webpackMode: "eager",
    webpackExports: ["a", "usedExports", "b"]
  */ "./dir12/a?3"
  ).then(f);
});

it("should contain webpackExports from module in weak mode", function () {
  require.resolve("./dir12/a?4");
  const f = module => {
    expect(module.usedExports).toEqual(["a", "b", "usedExports"]);
  };
  return import(
    /*
    webpackMode: "weak",
    webpackExports: ["a", "usedExports", "b"]
  */ "./dir12/a?4"
  ).then(f);
});

it("should not mangle webpackExports from module", function () {
  const f = module => {
    expect(module).toHaveProperty("longnameforexport");
  };
  return import(/* webpackExports: "longnameforexport" */ "./dir12/a?5").then(f);
});

it("should not mangle default webpackExports from module", function () {
  const f = module => {
    expect(module).toHaveProperty("default");
  };
  return import(/* webpackExports: "default" */ "./dir12/a?6").then(f);
});

it("should contain only webpackExports from module in context mode", function () {
  const x = "b";
  const f = module => {
    expect(module.usedExports).toEqual(["usedExports"]);
  };
  return import(/* webpackExports: "usedExports" */ `./dir13/${x}`).then(f);
});