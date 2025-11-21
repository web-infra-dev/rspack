const fs = require("node:fs")

console.log("line1")
console.log("line2")
console.log("line3")

it('should minimize *.bundle', () => {
  const content = fs.readFileSync(__filename, "utf-8");

  expect(content.split(/\r?\n/).length).toBe(1);
});