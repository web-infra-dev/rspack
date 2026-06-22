const value = require("./lib");

it("loads through the rspack runtime context require alias", () => {
  expect(value).toBe(42);
});
