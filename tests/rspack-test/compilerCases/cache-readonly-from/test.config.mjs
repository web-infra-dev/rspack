import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { runNodeCase } from "@rspack/test-tools/helper/node-case";

const scenario = new URL("./build.mjs", import.meta.url);
const profiles = {
  moduleProfile: "plain-node-v1",
  loaderProfile: "transform-v1",
};
const shared = ["dep.js", "lazy.js", "shared.js"];

function contents(directory) {
  return fs
    .readdirSync(directory, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => {
      const file = path.join(entry.parentPath, entry.name);
      return [
        path.relative(directory, file),
        createHash("sha256").update(fs.readFileSync(file)).digest("hex"),
      ];
    })
    .sort(([a], [b]) => a.localeCompare(b));
}

async function fixture(run) {
  const root = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "rspack-cache-reuse-")),
  );
  const files = {
    "app.js": 'module.exports = require("./shared.js");',
    "test.js":
      'module.exports = { ...require("./shared.js"), own: require("./own.js") };',
    "seed.js": 'module.exports = { own: require("./own.js") };',
    "shared.js":
      'exports.value = "__TRANSFORM__:" + require("./dep.js"); exports.file = __filename; exports.lazy = () => import("./lazy.js").then(m => m.default);',
    "dep.js": 'module.exports = "dep";',
    "own.js": 'module.exports = "consumer";',
    "lazy.js": 'module.exports = "lazy";',
    "aux.txt": "aux1",
    "build-dependency.json": '{"revision":1}',
    "loader.cjs": `const fs = require("node:fs"); const path = require("node:path");
      module.exports = function(source) {
        const aux = path.join(this.rootContext, "aux.txt");
        this.addDependency(aux);
        fs.appendFileSync(path.join(this.rootContext, "loads.jsonl"), path.basename(this.resourcePath) + "\\n");
        return source.replace("__TRANSFORM__", this.getOptions().tag + ":" + fs.readFileSync(aux, "utf8"));
      };`,
  };
  for (const [name, content] of Object.entries(files))
    fs.writeFileSync(path.join(root, name), content);
  const build = async (settings) => {
    fs.rmSync(path.join(root, "loads.jsonl"), { force: true });
    return runNodeCase(scenario, [
      root,
      Object.fromEntries(
        Object.entries(settings).filter(([, value]) => value !== undefined),
      ),
    ]);
  };
  try {
    await build({
      ...profiles,
      cache: "app-cache",
      entry: "app.js",
      producer: true,
    });
    const before = contents(path.join(root, "app-cache"));
    await run(root, build);
    assert.deepEqual(
      contents(path.join(root, "app-cache")),
      before,
      "source cache must remain byte-for-byte unchanged",
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
  {
    name: "modules",
    description:
      "should import compatible modules with fresh dependency IDs and persist them locally",
    async run() {
      await fixture(async (_root, build) => {
        await build({ ...profiles, cache: "test-cache", entry: "seed.js" });
        const result = await build({
          ...profiles,
          cache: "test-cache",
          source: "app-cache",
        });
        for (const name of shared)
          assert.ok(
            result.reused.includes(name),
            `${name}: ${JSON.stringify(result)}`,
          );
        assert.ok(result.reused.includes("own.js"));
        assert.deepEqual(result.loads, ["test.js"]);
        assert.equal(result.value, "one:aux1:dep");
        assert.equal(result.own, "consumer");
        assert.equal(result.lazy, "lazy");
        // No source configured: only the consumer's persisted cache is available.
        const warm = await build({ ...profiles, cache: "test-cache" });
        assert.deepEqual(warm.built, []);
        assert.deepEqual(warm.loads, []);
        assert.equal(warm.lazy, "lazy");
      });
    },
  },
  {
    name: "rstest-loaders",
    description:
      "should reuse app loader results while parsing modules with RstestPlugin",
    async run() {
      await fixture(async (root, build) => {
        const result = await build({
          loaderProfile: profiles.loaderProfile,
          cache: "rstest-cache",
          source: "app-cache",
          rstest: true,
        });
        for (const name of shared) {
          assert.ok(result.built.includes(name));
          assert.ok(!result.loads.includes(name));
        }
        assert.equal(result.file, path.join(root, "shared.js"));
        assert.equal(result.value, "one:aux1:dep");
        assert.equal(result.lazy, "lazy");
        const localLoaders = await build({
          loaderProfile: profiles.loaderProfile,
          cache: "rstest-cache",
          rstest: true,
          moduleCache: false,
        });
        assert.deepEqual(localLoaders.loads, []);
        assert.equal(localLoaders.file, path.join(root, "shared.js"));
        const changedProfile = await build({
          loaderProfile: "different-transforms",
          cache: "rstest-cache",
          source: "app-cache",
          rstest: true,
        });
        for (const name of shared)
          assert.ok(changedProfile.loads.includes(name));
      });
    },
  },
  {
    name: "invalidation",
    description:
      "should reject incompatible profiles, options, snapshots and source versions",
    async run() {
      await fixture(async (root, build) => {
        for (const [index, settings] of [
          { moduleProfile: "other", loaderProfile: "other" },
          { moduleProfile: undefined, loaderProfile: undefined },
          { sourceVersion: "wrong" },
          { tag: "two" },
          { source: "absent" },
        ].entries()) {
          const result = await build({
            ...profiles,
            cache: `miss-${index}`,
            source: "app-cache",
            ...settings,
          });
          for (const name of shared)
            assert.ok(
              result.loads.includes(name),
              `${name}: ${JSON.stringify(result)}`,
            );
          assert.equal(result.value, `${settings.tag ?? "one"}:aux1:dep`);
        }
        assert.equal(fs.existsSync(path.join(root, "absent")), false);
        const parser = await build({
          ...profiles,
          cache: "parser-cache",
          source: "app-cache",
          parser: { importMeta: false },
        });
        assert.ok(parser.built.includes("shared.js"));
        assert.ok(!parser.loads.includes("shared.js"));
        fs.writeFileSync(path.join(root, "aux.txt"), "aux2");
        const dependency = await build({
          ...profiles,
          cache: "dependency-cache",
          source: "app-cache",
        });
        for (const name of shared) assert.ok(dependency.loads.includes(name));
        assert.equal(dependency.value, "one:aux2:dep");
        fs.writeFileSync(path.join(root, "aux.txt"), "aux1");
        fs.writeFileSync(
          path.join(root, "shared.js"),
          'exports.value = "changed";',
        );
        const changed = await build({
          ...profiles,
          cache: "changed-cache",
          source: "app-cache",
        });
        assert.ok(changed.loads.includes("shared.js"));
        assert.equal(changed.value, "changed");
        // A globally invalid producer is ignored without resetting its database on disk.
        fs.writeFileSync(
          path.join(root, "build-dependency.json"),
          '{"revision":2}',
        );
        const invalid = await build({
          ...profiles,
          cache: "invalid-cache",
          source: "app-cache",
        });
        assert.ok(invalid.loads.includes("shared.js"));
      });
    },
  },
  {
    name: "overlapping-locations",
    description:
      "should reject overlapping source and destination cache locations",
    async run() {
      await fixture(async (_root, build) => {
        for (const [cache, source] of [
          ["app-cache", "app-cache"],
          ["app-cache/consumer", "app-cache"],
          ["container", "container/producer"],
        ]) {
          await assert.rejects(build({ cache, source }), /must not overlap/);
        }
      });
    },
  },
  {
    name: "promoted-build-dependencies",
    description:
      "should preserve producer build dependency invalidation after promotion",
    async run() {
      await fixture(async (root, build) => {
        await build({ ...profiles, cache: "test-cache", source: "app-cache" });
        fs.writeFileSync(
          path.join(root, "build-dependency.json"),
          '{"revision":2}',
        );
        const result = await build({ ...profiles, cache: "test-cache" });
        for (const name of shared) assert.ok(result.loads.includes(name));
      });
    },
  },
];
