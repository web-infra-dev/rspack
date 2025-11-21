const path = require("path");

it("should successfully emit foo.js", () => {
  expect(path.basename(__filename)).toBe("foo.js");
});
