it("should contain only one export from webpackExports from module", function () {
  return import(/* webpackExports: "usedExports" */ "./dir12/a?1").then(
    module => {
      expect(module.usedExports).toEqual(["usedExports"]);
    }
  );
});

it("should contain only webpackExports from module", function () {
  return import(
    /* webpackExports: ["a", "usedExports", "b"] */ "./dir12/a?2"
  ).then(module => {
    expect(module.usedExports).toEqual(["a", "b", "usedExports"]);
  });
});

it("should contain only webpackExports from module in eager mode", function () {
  return import(
    /*
    webpackMode: "eager",
    webpackExports: ["a", "usedExports", "b"]
  */ "./dir12/a?3"
  ).then(module => {
    expect(module.usedExports).toEqual(["a", "b", "usedExports"]);
  });
});

it("should contain webpackExports from module in weak mode", function () {
  require.resolve("./dir12/a?4");
  return import(
    /*
    webpackMode: "weak",
    webpackExports: ["a", "usedExports", "b"]
  */ "./dir12/a?4"
  ).then(module => {
    expect(module.usedExports).toEqual(["a", "b", "usedExports"]);
  });
});

it("should not mangle webpackExports from module", function () {
  return import(/* webpackExports: "longnameforexport" */ "./dir12/a?5").then(
    module => {
      expect(module).toHaveProperty("longnameforexport");
    }
  );
});

it("should not mangle default webpackExports from module", function () {
  return import(/* webpackExports: "default" */ "./dir12/a?6").then(
    module => {
      expect(module).toHaveProperty("default");
    }
  );
});

it("should contain only webpackExports from module in context mode", function () {
  const x = "b";
  return import(/* webpackExports: "usedExports" */ `./dir13/${x}`).then(
    module => {
      expect(module.usedExports).toEqual(["usedExports"]);
    }
  );
});