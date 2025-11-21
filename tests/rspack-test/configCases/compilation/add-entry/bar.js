const path = require("path");

it("should successfully emit bar.js", () => {
  expect(path.basename(__filename)).toBe("bar.js");
});

