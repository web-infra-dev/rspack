if (Math.random() < 0) {
  new Worker(
    /* webpackChunkName: "worker" */ new URL("./worker.js", import.meta.url),
  );
}

it("should resolve a modern-module worker URL from the output root", () => {
  const fs = require("fs");
  const path = require("path");
  const source = fs.readFileSync(
    path.join(__dirname, "js/main.js"),
    "utf-8",
  );

  expect(source).toContain(
    'new URL("../worker.bundle.js", import.meta.url)',
  );
});
