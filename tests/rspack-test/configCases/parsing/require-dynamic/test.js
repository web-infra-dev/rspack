const fs = require("fs");
const path  = require("path");

it("require a dynamical expression should preserve as-is when `requireDynamic` is disabled", () => {
  const code = fs.readFileSync(path.join(__dirname, "./bundle0.js"), 'utf-8');
  expect(code).toContain("require(dir)");
  expect(code).not.toContain("require('./other.js')");
  expect(code).toContain("require('./foo/' + dir + '.js')");
  expect(code).toContain("require(a + './foo/' + dir + '.js')");
  expect(code).toContain("require(dir ? './foo/' + dir + '.js' : './foo/nested' + dir + 'js')");
});
