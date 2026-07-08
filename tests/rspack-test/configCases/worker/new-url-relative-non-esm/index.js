if (Math.random() < 0) {
  new Worker(
    /* webpackChunkName: "worker" */ new URL("./worker.js", import.meta.url),
  );
}

it("should keep runtime worker URL output for non-ESM output", () => {
  const fs = require("fs");
  const path = require("path");
  const source = fs.readFileSync(path.join(__dirname, "main.js"), "utf-8");

  expect(source).toContain("__webpack_require__.p +");
  expect(source).toContain("__webpack_require__.u(");
  expect(source).not.toContain(
    'new URL("/public/worker.bundle.js", import.meta.url)',
  );
});
