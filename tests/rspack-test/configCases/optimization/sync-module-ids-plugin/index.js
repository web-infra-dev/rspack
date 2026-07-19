const a = require("./a");
const b = require("./b");

it("should sync module ids across compilers", () => {
  expect(a + b).toBe("ab");

  const fs = require("fs");
  const path = require("path");
  const ids = JSON.parse(
    fs.readFileSync(path.join(__dirname, "read-module-ids.json"), "utf-8")
  );

  expect(typeof ids["./a.js"]).toBe("string");
  expect(typeof ids["./b.js"]).toBe("string");

  const source = fs.readFileSync(
    path.join(__dirname, `bundle${__STATS_I__}.js`),
    "utf-8"
  );

  expect(source).toContain(`${JSON.stringify(ids["./a.js"])}(module)`);
  expect(source).toContain(`${JSON.stringify(ids["./b.js"])}(module)`);
});
