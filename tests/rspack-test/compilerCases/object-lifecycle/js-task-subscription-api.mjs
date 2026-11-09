import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { Worker, isMainThread, parentPort, threadId } from "node:worker_threads";
import { createGCTracker } from "./helpers.mjs";

const binding = createRequire(import.meta.resolve("@rspack/core"))("@rspack/binding");

if (isMainThread) {
  assert.throws(() => binding.getJsTaskSubscription(false), /runner has not been registered/);
  binding.registerJsTaskRunner(() => {});
  assert.throws(() => binding.getJsTaskSubscription(true), /role does not match/);
  const worker = new Worker(new URL(import.meta.url));
  const request = command => new Promise((resolve, reject) => {
    worker.once("message", message => message.error ? reject(new Error(message.error)) : resolve());
    worker.once("error", reject);
    worker.postMessage(command);
  });
  const tracker = createGCTracker();
  try {
    await request("missing");
    let first = binding.getJsTaskSubscription(false);
    let second = binding.getJsTaskSubscription(false);
    tracker.track(first, "first holder");
    tracker.track(second, "second holder");
    await request("attach");
    first = undefined;
    await tracker.waitForCollection("first holder");
    await request("active");
    second = undefined;
    await tracker.waitForCollection("second holder");
    await request("missing");
    let replacement = binding.getJsTaskSubscription(false);
    tracker.track(replacement, "replacement holder");
    await request("attach");
    replacement = undefined;
    await tracker.waitForCollection("replacement holder");
    await request("missing");
  } finally {
    await worker.terminate();
  }
} else {
  binding.registerJsTaskRunner(() => {}, threadId);
  parentPort.on("message", command => {
    try {
      if (command === "missing") {
        assert.throws(() => binding.getJsTaskSubscription(true), /Main JS task subscription is unavailable/);
        assert.throws(() => binding.beginJsTaskImport(), /subscription has closed/);
      } else {
        if (command === "attach") {
          binding.getJsTaskSubscription(true);
          binding.getJsTaskSubscription(true);
          assert.throws(() => binding.getJsTaskSubscription(false), /role does not match/);
        }
        const permit = binding.beginJsTaskImport();
        binding.endJsTaskImport(permit);
      }
      parentPort.postMessage({});
    } catch (error) {
      parentPort.postMessage({ error: error.stack });
    }
  });
}
