const path = require("path");

it("should successfully emit main.js", () => {
  expect(path.basename(__filename)).toBe("main.js");
});
