import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { rspack } from "@rspack/core";

export default [false, true].flatMap((retained) =>
  [false, true].flatMap((incremental) =>
    ["webpack", "rspack"].map((runtimeMode) => ({
      description: `updates or disposes a moved cached module (${retained}, ${incremental}, ${runtimeMode})`,
      options(context) {
        return {
          context: context.getDist("input"),
          entry: "./index.js",
          mode: "development",
          target: "web",
          devtool: false,
          incremental,
          experiments: { runtimeMode },
          output: {
            path: context.getDist("output"),
            filename: "[name].js",
            chunkFilename: "[name].js",
            publicPath: "/",
          },
          optimization: {
            moduleIds: "named",
            chunkIds: "named",
            concatenateModules: false,
            usedExports: false,
            splitChunks: {
              cacheGroups: {
                default: false,
                defaultVendors: false,
                extra: {
                  test: /[\\/]extra\.js$/,
                  name: "extra",
                  chunks: "all",
                  enforce: true,
                },
              },
            },
          },
          plugins: [new rspack.HotModuleReplacementPlugin()],
        };
      },
      async build(context, compiler) {
        const input = context.getDist("input");
        fs.mkdirSync(input, { recursive: true });
        const write = (name, source) =>
          fs.writeFileSync(path.join(input, name), source);
        write(
          "index.js",
          `
        globalThis.req = __webpack_require__; globalThis.hot = module.hot;
        globalThis.cache = require.cache; globalThis.factories = __webpack_modules__;
        require('./holder');
        globalThis.loadOther = () => import(/* webpackChunkName: 'other' */ './other');
        module.hot.accept();
      `,
        );
        write(
          "other.js",
          `module.exports = require('./dep'); ${retained ? "require('./mod');" : ""}`,
        );
        write("extra.js", "module.exports = 3;");
        const builds = [];
        for (let step = 0; step < 4; step++) {
          write(
            "holder.js",
            `${!retained || !step ? "require('./mod');" : ""} module.hot.accept();`,
          );
          write(
            "mod.js",
            `globalThis.value = ${retained || !step ? "require('./dep')" : "9"}; module.hot.accept();`,
          );
          write(
            "dep.js",
            `module.exports = ${step < 2 ? "2" : step === 2 ? "require('./extra')" : "4"};`,
          );
          const stats = await new Promise((resolve, reject) =>
            compiler.run(
              (error, stats) => (error ? reject(error) : resolve(stats)),
              {
                modifiedFiles: new Set(
                  ["holder.js", "mod.js", "dep.js"].map((name) =>
                    path.join(input, name),
                  ),
                ),
              },
            ),
          );
          assert.equal(stats.hasErrors(), false, stats.toString());
          builds.push(
            Object.fromEntries(
              stats.compilation
                .getAssets()
                .map((asset) => [asset.name, asset.source.source().toString()]),
            ),
          );
        }
        for (const mode of [
          "legacy",
          "idle",
          "loaded",
          "late-loaded",
          "pending",
          "abort",
          "self-accepted",
          "declined",
          "loaded-declined",
        ]) {
          let assets = builds[0];
          const sandbox = vm.createContext({
            self: {},
            console,
            Promise,
            setTimeout,
            clearTimeout,
          });
          vm.runInContext(assets["main.js"], sandbox);
          const { hot, cache, factories } = sandbox;
          const requests = [];
          let manifest;
          let deliverOther;
          sandbox.fetch = async (url) => {
            manifest = JSON.parse(assets[path.basename(url)]);
            if (mode === "legacy") delete manifest.changed;
            return { ok: true, json: async () => manifest };
          };
          sandbox.document = {
            getElementsByTagName: () => [],
            createElement: () => ({ setAttribute() {} }),
            head: {
              appendChild(script) {
                const name = path.basename(script.src);
                requests.push(name);
                assert(assets[name], `missing asset ${name}`);
                const deliver = () => {
                  vm.runInContext(mode === "late-loaded" && name === "other.js" ? builds[1][name] : assets[name], sandbox);
                  script.onload({ type: "load", target: script });
                };
                if (mode === "pending" && name === "other.js") deliverOther = deliver;
                else queueMicrotask(deliver);
              },
            },
          };
          const check = async (step) => {
            assets = builds[step];
            return hot.check(false);
          };
          await check(1);
          assert(
            !manifest.changed?.["./dep.js"],
            "moving alone is not a factory change",
          );
          await hot.apply();
          const oldModule = cache["./dep.js"];
          assert.equal(oldModule.exports, 2);
          let disposed = 0;
          oldModule.hot.dispose(() => disposed++);
          if (mode.startsWith("loaded")) await sandbox.loadOther();
          // Start with the old import's chunk list, then deliver other.js after apply.
          const pending = mode === "pending" && sandbox.loadOther();
          const oldFactory = factories["./dep.js"];
          if (mode === "self-accepted") oldModule.hot.accept();
          if (mode.endsWith("declined")) oldModule.hot.decline();
          requests.length = 0;
          await check(2);
          if (mode !== "legacy")
            assert.deepEqual(manifest.changed["./dep.js"], ["other"]);
          assert.equal(
            cache["./dep.js"],
            oldModule,
            "check must not dispose the running module",
          );
          assert.equal(factories["./dep.js"], oldFactory);
          if (mode === "late-loaded") await sandbox.loadOther();
          if (mode === "abort") {
            hot.decline();
            hot.invalidate();
          }
          if (mode === "abort" || mode === "loaded-declined") {
            await assert.rejects(hot.apply(), /declin/);
            assert.equal(cache["./dep.js"], oldModule);
            assert.equal(factories["./dep.js"], oldFactory);
            assert.equal(disposed, 0);
            continue;
          }
          await hot.apply();
          if (pending) {
            deliverOther();
            assert.equal((await pending).default, 3);
            continue;
          }
          if (mode === "legacy") {
            assert.equal(disposed, 0);
            assert.equal(
              (await sandbox.loadOther()).default,
              2,
              "without notification a fresh chunk still uses stale cache",
            );
            continue;
          }
          assert.equal(disposed, 1);
          if (mode !== "loaded" && mode !== "late-loaded") {
            assert.equal(cache["./dep.js"], undefined);
            assert.notEqual(
              factories["./dep.js"],
              oldFactory,
              "the old factory must not remain executable",
            );
            assert(
              !requests.some(
                (name) =>
                  name.startsWith("other.") || name.startsWith("extra."),
              ),
            );
            assert.equal(sandbox.value, retained ? 2 : 9);
          } else {
            assert.equal(sandbox.req("./dep.js"), 3);
            if (retained) assert.equal(sandbox.value, 3);
          }
          assert.equal((await sandbox.loadOther()).default, 3);
          // Once the full chunk is installed, subsequent updates take the ordinary path.
          requests.length = 0;
          await check(3);
          await hot.apply();
          assert.equal(sandbox.req("./dep.js"), 4);
          assert(!requests.some((name) => name.endsWith(".full.js")));
          assert.equal(hot.status(), "idle");
        }
      },
    })),
  ),
);
