if (Math.random() < 0) {
  new Worker(
    /* webpackChunkName: "worker" */ new URL("./worker.js", import.meta.url),
  );
}

it("should emit a static worker URL with publicPath", () => {
  const fs = require("fs");
  const path = require("path");
  const source = fs.readFileSync(path.join(__dirname, "main.js"), "utf-8");

  expect(source).toContain(
    'new URL("/public/worker.bundle.js", import.meta.url)',
  );
  expect(source).not.toContain("__webpack_require__.p +");
});
