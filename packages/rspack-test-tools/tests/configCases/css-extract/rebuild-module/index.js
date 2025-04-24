const fs = require("fs");
const path = require("path");

it("should rebuild module", async () => {
  await import("./render").then((exports) => {
    exports.render();
  });

  const css = fs.readFileSync(path.join(__dirname, "bundle.css"), "utf-8");
  expect(css).toContain("red");
});
