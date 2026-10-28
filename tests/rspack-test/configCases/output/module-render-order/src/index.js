it("renders modules in context-relative identifier order", () => {
  expect(require("../z")).toBe("render-order:outside");
  expect(require("./a")).toBe("render-order:inside");
  expect(require("../loader.cjs!./loaded")).toBe("render-order:loaded");
});

it("renders async chunk modules in the same order", async () => {
  const { default: values } = await import(
    /* webpackChunkName: "async" */ "./async"
  );
  expect(values).toEqual([
    "render-order:outside",
    "render-order:inside",
    "render-order:loaded",
  ]);
});
