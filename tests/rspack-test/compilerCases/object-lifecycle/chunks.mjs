import assert from "node:assert/strict";
import { rspack } from "@rspack/core";
import { createFsFromVolume, Volume } from "memfs";
import {
  closeCompiler,
  createGCTracker,
  runCompiler,
} from "@rspack/test-tools/helper/lifecycle";

export default async function run() {
  const gcTracker = createGCTracker();
  const fixtureDir = import.meta.dirname;
  let observedChunks = false;
  let retainedCollections;

  let compiler = rspack({
    context: fixtureDir,
    mode: "development",
    entry: "./entry.js",
    output: {
      path: "/",
      filename: "bundle.js",
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.compilation.tap(
            "TsfnLifecycleChunks",
            (compilation) => {
              compilation.hooks.afterProcessAssets.tap(
                "TsfnLifecycleChunks",
                () => {
                  const chunks = Array.from(compilation.chunks);
                  const chunkGroups = Array.from(compilation.chunkGroups);

                  if (chunks.length === 0) {
                    throw new Error("expected at least one chunk");
                  }

                  if (chunkGroups.length === 0) {
                    throw new Error("expected at least one chunk group");
                  }

                  let chunk = chunks[0];
                  let chunkGroup = chunkGroups[0];
                  let filesView = chunk.files;
                  retainedCollections = [
                    ["chunk files", chunk._files],
                    ["chunk auxiliary files", chunk._auxiliaryFiles],
                    ["chunk runtime", chunk._runtime],
                  ];
                  for (const [name, collection] of retainedCollections) {
                    gcTracker.track(collection, name);
                    const values = collection.values();
                    assert.equal(collection.values(), values);
                    assert.equal(collection.size(), values.length);
                    gcTracker.track(values, `${name} array`);
                  }
                  Array.from(filesView);

                  gcTracker.track(chunk, "chunk");
                  gcTracker.track(chunkGroup, "chunk group");
                  gcTracker.track(filesView, "chunk files view");

                  filesView = null;
                  chunk = null;
                  chunkGroup = null;
                  chunks.length = 0;
                  chunkGroups.length = 0;
                  observedChunks = true;
                },
              );
            },
          );
        },
      },
    ],
  });
  compiler.outputFileSystem = createFsFromVolume(new Volume());

  let stats;
  try {
    stats = await runCompiler(compiler);
    if (!observedChunks) {
      throw new Error("chunks were not observed");
    }

    gcTracker.track(compiler, "chunk compiler");
  } finally {
    await closeCompiler(compiler);
  }

  stats = null;
  compiler = null;

  await gcTracker.waitForCollection("chunk compiler");
  // Retaining a collection must neither keep its owner alive nor bypass the
  // owner check when its array has already been cached.
  // Keep collection locals out of this async frame before the GC assertions.
  retainedCollections.forEach(([, collection]) => {
    assert.throws(() => collection.size(), /Unable to access compilation/);
    assert.throws(() => collection.has("bundle.js"), /Unable to access compilation/);
    assert.throws(() => collection.values(), /Unable to access compilation/);
  });
  retainedCollections = null;
  await gcTracker.waitForCollection("chunk");
  await gcTracker.waitForCollection("chunk group");
  await gcTracker.waitForCollection("chunk files view");
  for (const name of ["chunk files", "chunk auxiliary files", "chunk runtime"]) {
    await gcTracker.waitForCollection(name);
    await gcTracker.waitForCollection(`${name} array`);
  }
}
