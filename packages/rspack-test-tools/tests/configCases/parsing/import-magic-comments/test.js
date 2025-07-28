const fs = require("fs");
const path  = require("path");

it("magic comments should preserve as-is when `magicComments` is disabled", () => {
  const code = fs.readFileSync(path.join(__dirname, "./index.js"), 'utf-8');
  expect(code).toContain(`/* webpackChunkName: "my-chunk-name" */`);
});
