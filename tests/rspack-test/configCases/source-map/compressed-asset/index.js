const message = "原始 source map 内容 😀";

it("emits ordinary JSON from the compressed map asset", () => {
  const fs = require("node:fs");
  const path = require("node:path");
  const code = fs.readFileSync(__filename, "utf8");
  const filename = /^\/\/# sourceMappingURL=(.*)$/m.exec(code)[1];
  const map = JSON.parse(
    fs.readFileSync(path.join(__dirname, filename), "utf8"),
  );
  expect(map.version).toBe(3);
  expect(map.sourcesContent.some((content) => content.includes(message))).toBe(
    true,
  );
});
