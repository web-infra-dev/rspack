const context = require.context("./modules", false, /\.js$/);

it("keeps require.context separate from rspack runtime context", () => {
  expect(context("./a.js").value).toBe("a");
  expect(context.keys()).toEqual(["./a.js"]);
});
