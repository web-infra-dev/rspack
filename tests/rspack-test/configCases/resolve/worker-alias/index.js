function createWorker() {
  return new Worker(new URL('./worker.js', import.meta.url));
}

new Map(), createWorker;

const fs = require("fs");
const path = require("path");

it("should compile", () => {
  const worker = fs.readFileSync(path.resolve(__dirname, "worker.js"), "utf-8");
  expect(worker).toContain("this is a fake node_modules");
});
