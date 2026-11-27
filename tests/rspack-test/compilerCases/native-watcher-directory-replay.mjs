import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve("@rspack/core"))(
  "@rspack/binding",
);

async function waitFor(predicate) {
  const deadline = Date.now() + 5000;
  while (!predicate() && Date.now() < deadline) await sleep(10);
  expect(predicate()).toBe(true);
}

function fixture() {
  return fs.mkdtempSync(
    path.join(fs.realpathSync(os.tmpdir()), "rspack-watch-replay-"),
  );
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [
  {
    name: "close-from-ignored",
    description:
      "can close a directory scan while its JavaScript ignored predicate is running",
    async run() {
      const root = fixture();
      const file = path.join(root, "file.js");
      fs.writeFileSync(file, "");
      let closing;
      const watcher = new NativeWatcher({
        ignored(name) {
          if (name === file && !closing) {
            closing = watcher.close();
            Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 150);
          }
          return false;
        },
      });
      try {
        watcher.watch(
          [[], []],
          [[root], []],
          [[], []],
          BigInt(Date.now() + 60000),
          () => {},
          () => {},
        );
        await waitFor(() => Boolean(closing));
        await closing;
      } finally {
        await (closing ?? watcher.close());
        fs.rmSync(root, { recursive: true, force: true });
      }
    },
  },
];
