const fs = require("fs");
const path  = require("path");

it("use require as a expression should preserve as-is when `requireAsExpression` is disabled", () => {
  const code = fs.readFileSync(path.join(__dirname, "./bundle0.js"), 'utf-8');
  expect(code).not.toContain("require('./other.js')");
  expect(code).toContain("resolve1('./other.js')");
  expect(code).toContain(`lazyFn('./other.js', require)`);
});
