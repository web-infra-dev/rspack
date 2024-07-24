it("compatibility plugin", async () => {
  const f = require("./a.js");
  expect(f(1)).toBe(2);
});
