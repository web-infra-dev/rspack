if (Math.random() < 0) {
  new Worker(
    /* webpackChunkName: "worker" */ new URL("./worker.js", import.meta.url),
  );
}

it("should resolve a relative workerPublicPath from the output root", () => {
  const fs = require("fs");
  const path = require("path");
  const source = fs.readFileSync(
    path.join(__dirname, "js/main.js"),
    "utf-8",
  );

  expect(source).toContain(
    'new URL("../workers/worker.bundle.js", import.meta.url)',
  );
  expect(source).not.toContain("/public/worker.bundle.js");
});
