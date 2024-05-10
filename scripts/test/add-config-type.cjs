const glob = require("glob");
const fs = require("fs");
const path = require("path");

const testRoot = path.resolve(__dirname, "../../packages/rspack-test-tools/tests");
const files = glob.sync("**/webpack.config.js", {
  cwd: testRoot
});
const JSDOC_TYPE_COMMENT = `/** @type {import("@rspack/core").Configuration} */`;

for (let file of files) {
  const absolutePath = path.resolve(testRoot, file);
  let content = fs.readFileSync(absolutePath, 'utf-8');
  if (content.includes("@type")) {
    continue;
  }
  if (!content.includes("module.exports = {")) {
    continue;
  }
  const index = content.indexOf("module.exports = {");
  if (index === 0) {
    content = `${JSDOC_TYPE_COMMENT}\n${content}`;
  } else if (content[index - 1] === "\n") {
    let prevLineStart = index - 2;
    while (content[prevLineStart] !== "\n" && prevLineStart > 0) {
      prevLineStart--;
    }
    const line = content.slice(prevLineStart, index - 1);
    if (line.includes("*/")) {
      continue;
    } else {
      content = `${content.slice(0, index - 1)}\n${JSDOC_TYPE_COMMENT}\n${content.slice(index)}`;
    }
  }
  fs.writeFileSync(absolutePath, content, 'utf-8');
}

