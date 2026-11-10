import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import sourceMap from "source-map";
import {
  build, copyProject, expectColdEquality, expectEqualOutput,
  jsxSource, swcOptions, swcRule, withProjects, writeProject
} from "./helpers.mjs";

// Avoid making this fixture module itself look like it carries an inline source map to
// Node's stack-trace source-map reader. Generated test resources still use the exact marker.
const sourceMappingURL = "sourceMapping" + "URL=";

const automaticOptions = {
  ...swcOptions,
  jsc: {
    target: "es2022",
    parser: { syntax: "typescript", tsx: true, decorators: true },
    transform: { react: { runtime: "automatic", development: true } }
  }
};
const autoMapOptions = { ...swcOptions };
delete autoMapOptions.inputSourceMap;
const map = (name) => ({
  version: 3, sources: [name], sourcesContent: ["export const original = 1;"],
  names: [], mappings: "AAAA"
});
const writeMap = (root, name, original) => fs.writeFileSync(
  path.join(root, "src", name), JSON.stringify(map(original))
);
function expectMapped(result, name) {
  expect(JSON.parse(result.map).sources.some(source => source.endsWith(name))).toBe(true);
}
function installLoader(root, name) {
  // Resolve the existing test dependency; temporary projects do not install packages.
  const source = fs.readFileSync(new URL(`./${name}.cjs`, import.meta.url), "utf-8")
    .replace("require('source-map')", `require(${JSON.stringify(fileURLToPath(import.meta.resolve("source-map")))})`);
  fs.writeFileSync(path.join(root, `${name}.cjs`), source);
  return `./${name}.cjs`;
}
async function expectWarmAndUncached(first, root, rules, configuration, expectedBuilt = []) {
  const warm = await build(root, rules, undefined, configuration);
  expect(warm.built).toEqual(expectedBuilt);
  expectEqualOutput(warm, first);
  expectEqualOutput(await build(root, rules, false, configuration), first);
}

// A source map would embed the authored text and could hide what the literal became.
const withoutMaps = { devtool: false };
const targeting = target => ({ ...swcOptions, jsc: { ...swcOptions.jsc, target } });
const inTemplate = text => JSON.stringify(text).slice(1, -1);
const authoredSpellings = [
  {
    name: "es5-template",
    file: "spelled.jsx",
    target: "es5",
    source: (dir, file) => `export const fixed = \`${dir}\${"${file}"}\`;`
  },
  {
    name: "es2020-template",
    file: "spelled.jsx",
    target: "es2020",
    source: (dir, file) => `export const fixed = \`${dir}\${"${file}"}\`;`
  },
  {
    name: "es5-tagged-template",
    file: "spelled.jsx",
    target: "es5",
    source: (dir, file) => `const first = strings => strings[0];
export const fixed = first\`${dir}${file}\`;`
  },
  {
    name: "es5-template-numeric-literal",
    file: "v1.jsx",
    target: "es5",
    source: dir => `export const fixed = \`${dir}v\${1}.jsx\`;`
  },
  {
    name: "enum-string-initializer",
    file: "spelled.tsx",
    target: "es2020",
    source: (dir, file) => `enum Paths { File = "${dir}" + "${file}" }
export const fixed = Paths.File;`
  },
  {
    name: "enum-exponent-format",
    file: "v1000.tsx",
    target: "es2020",
    source: dir => `enum Paths { File = "${dir}v" + 1e3 + ".tsx" }
export const fixed = Paths.File;`
  },
  {
    name: "enum-hex-format",
    file: "v16.tsx",
    target: "es2020",
    source: dir => `enum Paths { File = "${dir}v" + 0x10 + ".tsx" }
export const fixed = Paths.File;`
  },
  {
    name: "enum-arithmetic",
    file: "v3.tsx",
    target: "es2020",
    source: dir => `enum Paths { File = "${dir}v" + (1 + 2) + ".tsx" }
export const fixed = Paths.File;`
  },
  {
    name: "enum-auto-increment",
    file: "v1.tsx",
    target: "es2020",
    source: dir => `enum Paths { First, Second, File = \`${dir}v\${Second}.tsx\` }
export const fixed = Paths.File;`
  },
  {
    name: "enum-large-number",
    file: "v1e+21.tsx",
    target: "es2020",
    source: dir => `enum Paths { File = "${dir}v" + 1e21 + ".tsx" }
export const fixed = Paths.File;`
  },
  {
    name: "enum-not-a-number",
    file: "vNaN.tsx",
    target: "es2020",
    source: dir => `enum Paths { File = "${dir}v" + NaN + ".tsx" }
export const fixed = Paths.File;`
  },
  {
    name: "enum-joined-surrogates",
    file: "😀.tsx",
    target: "es2020",
    source: dir => `enum Paths { File = "${dir}\\ud83d" + "\\ude00.tsx" }
export const fixed = Paths.File;`
  }
];

export default [
  ...authoredSpellings.map(({ name, file, target, source }) => ({
    name: `authored-spelling-${name}`,
    description: "keeps a path spelled by authored literals fixed when a relocated build could otherwise rewrite it",
    async run() {
      await withProjects(async (writer, reader) => {
        const dir = inTemplate(path.join(writer, "src") + path.sep);
        writeProject(writer, { [file]: source(dir, file) });
        const rules = [swcRule(/\.(jsx|tsx)$/, targeting(target))];
        const fixed = path.join(writer, "src", file);
        const first = await build(writer, rules, undefined, withoutMaps);
        expect(first.values[0].fixed).toBe(fixed);
        copyProject(writer, reader);
        const moved = await build(reader, rules, undefined, withoutMaps);
        expect(moved.values[0].fixed).toBe(fixed);
        await expectColdEquality(moved, reader, rules, withoutMaps);
      });
    }
  })),
  {
    name: "short-fragments-keep-certification",
    description: "keeps reusing a module whose literals and numeric enum cover only part of the path while relocating its JSX filename",
    async run() {
      await withProjects(async (writer, reader) => {
        writeProject(writer, {
          "component.tsx": jsxSource(`enum Flags { A = 1 << 0, B = 1 << 1 }
export const parts = ["/", "src", ".tsx", \`/\${"src"}/\`, Flags.B, true, null, 1];`)
        });
        const rules = [swcRule(/\.tsx$/, targeting("es5"))];
        const first = await build(writer, rules, undefined, withoutMaps);
        expect(first.built).toEqual(["component.tsx"]);
        copyProject(writer, reader);
        const moved = await build(reader, rules, undefined, withoutMaps);
        expect(moved.built).toEqual([]);
        expect(moved.values[0].value.fileName).toBe(path.join(reader, "src", "component.tsx"));
        await expectColdEquality(moved, reader, rules, withoutMaps);
      });
    }
  },
  ...[false, true].map(portableOutput => ({
    name: `async-map-filesystem-${portableOutput ? "portable" : "default"}`,
    description: "acquires absent and virtual-only adjacent input maps through async filesystem callbacks on native and WASI bindings",
    async run() {
      if (process.env.NAPI_RS_FORCE_WASI) {
        // The generated binding loader can otherwise retain its native binding if WASI
        // initialization fails. A forced-WASI run must exercise the actual WASI exports.
        const require = createRequire(import.meta.url);
        const requireCore = createRequire(require.resolve("@rspack/core"));
        expect(requireCore("@rspack/binding")).toBe(require("../../../../crates/node_binding/rspack.wasi.cjs"));
      }
      await withProjects(async writer => {
        writeProject(writer, {
          "absent.jsx": jsxSource(),
          "virtual.jsx": jsxSource(),
          "denied.jsx": jsxSource(),
          "unknown.jsx": jsxSource(),
          "marked.jsx": `${jsxSource()}//# ${sourceMappingURL}marked-original.map\n`
        });
        writeMap(writer, "marked-original.map", "marked-original.tsx");
        const markedMap = path.join(writer, "src", "marked-original.map");
        const absentMap = path.join(writer, "src", "absent.jsx.map");
        const virtualMap = path.join(writer, "src", "virtual.jsx.map");
        const deniedMap = path.join(writer, "src", "denied.jsx.map");
        const unknownMap = path.join(writer, "src", "unknown.jsx.map");
        expect(fs.existsSync(absentMap)).toBe(false);
        expect(fs.existsSync(virtualMap)).toBe(false);
        const probes = [];
        const reads = [];
        const configuration = {
          experiments: { useInputFileSystem: [/\.map$/] },
          configureCompiler(compiler) {
            const original = compiler.inputFileSystem;
            const replacement = Object.create(original);
            replacement.stat = (file, ...args) => {
              probes.push(file);
              // Both stat and readFile complete asynchronously on the JS thread. The map
              // exists only in this configured filesystem, never on the physical disk.
              // A misleading message must never make a permission or uncoded error
              // look like a missing file. Only the original error's code is authoritative.
              if (file === deniedMap || file === unknownMap) {
                const error = new Error("ENOENT: no such file or directory (audited message only)");
                if (file === deniedMap) error.code = "EACCES";
                return setImmediate(() => args.at(-1)(error));
              }
              setImmediate(() => original.stat(
                file === virtualMap ? path.join(writer, "src", "virtual.jsx") : file,
                ...args
              ));
            };
            replacement.readFile = (file, ...args) => {
              reads.push(file);
              if (file === virtualMap) {
                return setImmediate(() => args.at(-1)(null, Buffer.from(JSON.stringify(map("virtual-only-original.tsx")))));
              }
              return original.readFile(file, ...args);
            };
            compiler.inputFileSystem = replacement;
          }
        };
        const { rspackExperiments, ...options } = swcOptions;
        const rules = [swcRule(/\.jsx$/, {
          ...options,
          inputSourceMap: true,
          ...(portableOutput ? { rspackExperiments } : {})
        })];
        const result = await build(writer, rules, false, configuration);
        expect(result.values.map(entry => entry.value.title)).toEqual(["portable", "portable", "portable", "portable", "portable"]);
        expectMapped(result, "absent.jsx");
        expectMapped(result, "virtual-only-original.tsx");
        // The URL ends at the line break; the break is not part of the path.
        expectMapped(result, "marked-original.tsx");
        const dependencies = [...result.fileDependencies, ...result.missingDependencies];
        expect(dependencies.filter(file => /[\r\n]/.test(file))).toEqual([]);
        if (portableOutput) {
          expect(result.fileDependencies).toContain(virtualMap);
          expect(result.fileDependencies).toContain(markedMap);
          expect(result.missingDependencies).toContain(absentMap);
          expect(result.fileDependencies).not.toContain(absentMap);
          for (const uncertainMap of [deniedMap, unknownMap]) {
            expect(result.fileDependencies).toContain(uncertainMap);
            expect(result.missingDependencies).not.toContain(uncertainMap);
          }
        } else {
          // Without certification the loader records no map candidates, as before.
          expect(dependencies.filter(file => file.endsWith(".map"))).toEqual([]);
        }
        for (const uncertainMap of [deniedMap, unknownMap]) {
          expect(probes.filter(file => file === uncertainMap)).toHaveLength(1);
          expect(reads).not.toContain(uncertainMap);
        }
        expect(probes.filter(file => file === absentMap)).toHaveLength(1);
        expect(probes.filter(file => file === virtualMap)).toHaveLength(1);
        expect(reads.filter(file => file === virtualMap)).toHaveLength(1);
        expect(reads).not.toContain(absentMap);
        expect(fs.existsSync(virtualMap)).toBe(false);
      });
    }
  })),
  {
    name: "automatic-tsx-legacy-decorators",
    description: "certifies explicit TypeScript legacy decorators and real automatic JSX elements/fragments at ES2022",
    async run() {
      await withProjects(async (writer, reader) => {
        writeProject(writer, { "component.tsx": `
const calls: string[] = [];
function audited(target: object, key: string, descriptor: PropertyDescriptor) {
  if (typeof target !== "object" || typeof key !== "string" || !descriptor) {
    throw new Error("expected the legacy decorator calling convention");
  }
  calls.push(key);
  if (descriptor.value) {
    const method = descriptor.value;
    descriptor.value = function(...args: unknown[]) { return method.apply(this, args) + 1; };
  } else {
    const getter = descriptor.get!;
    descriptor.get = function() { return getter.call(this) + 2; };
  }
  return descriptor;
}
class Model {
  @audited method() { return 5; }
  @audited get label() { return 7; }
}
const model = new Model();
export const decorated = [calls, model.method(), model.label];
export const value = <><section title="automatic"><span>nested</span></section><aside /></>;
` });
        const rules = [swcRule(/\.tsx$/, automaticOptions)];
        const first = await build(writer, rules);
        expect(first.values[0].decorated).toEqual([["method", "label"], 6, 9]);
        expect(first.values[0].value.type).toBe("fragment");
        expect(first.values[0].value.children.map(child => child.type)).toEqual(["section", "aside"]);
        expect(first.values[0].value.children[0].children[0].type).toBe("span");
        await expectWarmAndUncached(first, writer, rules);
        copyProject(writer, reader);
        const moved = await build(reader, rules);
        expect(moved.built).toEqual([]);
        expect(moved.values[0].decorated).toEqual(first.values[0].decorated);
        for (const element of [moved.values[0].value.children[0], moved.values[0].value.children[1], moved.values[0].value.children[0].children[0]]) {
          expect(element.fileName).toBe(path.join(reader, "src", "component.tsx"));
          expect(element.line).toBeGreaterThan(0);
          expect(element.column).toBeGreaterThan(0);
        }
        await expectWarmAndUncached(moved, reader, rules);
        await expectColdEquality(moved, reader, rules);
      });
    }
  },
  {
    name: "same-line-filename-map-columns",
    description: "repairs decoded UTF-16 map columns after a generated filename on the same line across unequal Unicode roots",
    async run() {
      await withProjects(async (writer, reader) => {
        writeProject(writer, { "component.jsx": jsxSource() });
        const rule = swcRule(/\.jsx$/);
        rule.use.unshift(installLoader(writer, "compact-loader"));
        const rules = [rule];
        const first = await build(writer, rules);
        await expectWarmAndUncached(first, writer, rules);
        copyProject(writer, reader);
        const moved = await build(reader, rules);
        expect(moved.built).toEqual([]);
        for (const [result, root] of [[first, writer], [moved, reader]]) {
          const source = result.sources["component.jsx"];
          expect(source).not.toContain("\n");
          const literal = /fileName:\s*("(?:\\.|[^"\\])*")/.exec(source);
          expect(literal).not.toBeNull();
          expect(JSON.parse(literal[1])).toBe(path.join(root, "src", "component.jsx"));
          const filename = literal.index;
          expect(source.indexOf('"after the generated filename"')).toBeGreaterThan(filename);
          const index = result.output.indexOf('"after the generated filename"');
          expect(index).toBeGreaterThan(0);
          const prefix = result.output.slice(0, index);
          const consumer = await new sourceMap.SourceMapConsumer(JSON.parse(result.map));
          try {
            const original = consumer.originalPositionFor({
              line: prefix.split("\n").length,
              column: prefix.length - prefix.lastIndexOf("\n") - 1
            });
            expect(original.source).toMatch(/\/component\.jsx$/);
            expect(original.line).toBe(4);
            expect(original.column).toBe(21);
          } finally { consumer.destroy?.(); }
        }
        await expectWarmAndUncached(moved, reader, rules);
        await expectColdEquality(moved, reader, rules);
      });
    }
  },
  ...[undefined, true].map(inputSourceMap => ({
    name: inputSourceMap ? "automatic-map-true" : "automatic-map-default",
    description: "certifies an actually absent adjacent map with a missing dependency and invalidates on appearance, changes and deletion",
    async run() {
      await withProjects(async (writer, reader) => {
        writeProject(writer, { "component.jsx": jsxSource() });
        const options = { ...autoMapOptions, ...(inputSourceMap ? { inputSourceMap } : {}) };
        const rules = [swcRule(/\.jsx$/, options)];
        // Missing dependencies deliberately prevent minimal loader-chain storage. Module
        // cache reuse still must work; do not add maps to that cache's etag or weaken its rules.
        rules[0].use[0].cache = true;
        const first = await build(writer, rules);
        expect(first.missingDependencies).toContain(path.join(writer, "src", "component.jsx.map"));
        await expectWarmAndUncached(first, writer, rules);
        copyProject(writer, reader);
        const moved = await build(reader, rules);
        expect(moved.built).toEqual([]);
        expect(moved.missingDependencies).toContain(path.join(reader, "src", "component.jsx.map"));
        await expectWarmAndUncached(moved, reader, rules);
        await expectColdEquality(moved, reader, rules);
        writeMap(reader, "component.jsx.map", "appeared.tsx");
        const appeared = await build(reader, rules);
        expect(appeared.built).toEqual(["component.jsx"]);
        expect(appeared.fileDependencies).toContain(path.join(reader, "src", "component.jsx.map"));
        expectMapped(appeared, "appeared.tsx");
        await expectWarmAndUncached(appeared, reader, rules);
        writeMap(reader, "component.jsx.map", "changed-longer-name.tsx");
        const changed = await build(reader, rules);
        expect(changed.built).toEqual(["component.jsx"]);
        expectMapped(changed, "changed-longer-name.tsx");
        expect(changed.map).not.toBe(appeared.map);
        expectEqualOutput(await build(reader, rules, false), changed);
        fs.unlinkSync(path.join(reader, "src", "component.jsx.map"));
        const deleted = await build(reader, rules);
        expect(deleted.built).toEqual(["component.jsx"]);
        expectEqualOutput(deleted, moved);
        await expectWarmAndUncached(deleted, reader, rules);
      });
    }
  })),
  {
    name: "map-fallback-priority",
    description: "tracks the missing last-marker candidate before an adjacent fallback and invalidates when either map changes priority",
    async run() {
      await withProjects(async (writer, reader) => {
        writeProject(writer, {
          "component.jsx": jsxSource() + `//# ${sourceMappingURL}ignored.map\n//# ${sourceMappingURL}preferred.map`
        });
        writeMap(writer, "component.jsx.map", "fallback.tsx");
        const rules = [swcRule(/\.jsx$/, autoMapOptions)];
        const first = await build(writer, rules);
        expectMapped(first, "fallback.tsx");
        expect(first.missingDependencies).toContain(path.join(writer, "src", "preferred.map"));
        expect(first.missingDependencies).not.toContain(path.join(writer, "src", "ignored.map"));
        expect(first.fileDependencies).toContain(path.join(writer, "src", "component.jsx.map"));
        await expectWarmAndUncached(first, writer, rules);
        copyProject(writer, reader);
        const moved = await build(reader, rules);
        expect(moved.built).toEqual(["component.jsx"]);
        await expectWarmAndUncached(moved, reader, rules);
        writeMap(reader, "preferred.map", "preferred.tsx");
        const preferred = await build(reader, rules);
        expect(preferred.built).toEqual(["component.jsx"]);
        expectMapped(preferred, "preferred.tsx");
        expectEqualOutput(await build(reader, rules, false), preferred);
        fs.unlinkSync(path.join(reader, "src", "preferred.map"));
        const fallback = await build(reader, rules);
        expect(fallback.built).toEqual(["component.jsx"]);
        expectEqualOutput(fallback, moved);
        writeMap(reader, "component.jsx.map", "changed-fallback.tsx");
        const changed = await build(reader, rules);
        expect(changed.built).toEqual(["component.jsx"]);
        expectMapped(changed, "changed-fallback.tsx");
        fs.unlinkSync(path.join(reader, "src", "component.jsx.map"));
        const absent = await build(reader, rules);
        expect(absent.built).toEqual(["component.jsx"]);
        expect(absent.missingDependencies).toEqual(expect.arrayContaining([
          path.join(reader, "src", "preferred.map"), path.join(reader, "src", "component.jsx.map")
        ]));
        await expectColdEquality(absent, reader, rules);
      });
    }
  },
  ...[false, true].map(certified => ({
    name: certified ? "certified-map-and-data-producer" : "uncertified-map-and-data-producer",
    description: "composes audited incoming-map and additional-data authority and preserves opaque data through SWC",
    async run() {
      await withProjects(async (writer, reader) => {
        writeProject(writer, { "map.jsx": jsxSource(), "data.jsx": jsxSource(), "both.jsx": jsxSource() });
        const producer = installLoader(writer, "producer-loader");
        const consumer = installLoader(writer, "consumer-loader");
        const rules = ["map", "data", "both"].map(name => {
          const rule = swcRule(new RegExp(`${name}\\.jsx$`), autoMapOptions);
          if (name !== "map") rule.use.unshift(consumer);
          rule.use.push({ loader: producer, options: { certified, map: name !== "data", data: name !== "map" } });
          return rule;
        });
        const first = await build(writer, rules);
        await expectWarmAndUncached(first, writer, rules);
        copyProject(writer, reader);
        const moved = await build(reader, rules);
        expect(moved.built).toEqual(certified ? [] : ["both.jsx", "data.jsx", "map.jsx"]);
        for (const entry of moved.values.filter(entry => entry.metadata)) {
          expect(entry.metadata.preserved).toBe("producer-data");
          expect(entry.metadata.resource).toBe(entry.value.fileName);
        }
        await expectWarmAndUncached(moved, reader, rules);
        await expectColdEquality(moved, reader, rules);
      });
    }
  })),
  {
    name: "resource-map-provenance",
    description: "withholds aggregate authority for extractSourceMap maps acquired outside certified loader invocations",
    async run() {
      await withProjects(async (writer, reader) => {
        const fixed = "https://example.invalid/fixed-original.tsx";
        const inline = Buffer.from(JSON.stringify(map(fixed))).toString("base64");
        writeProject(writer, { "resource.jsx": jsxSource() + `//# ${sourceMappingURL}data:application/json;base64,${inline}` });
        const rule = swcRule(/\.jsx$/, autoMapOptions);
        rule.extractSourceMap = true;
        const rules = [rule];
        const first = await build(writer, rules);
        expectMapped(first, fixed);
        await expectWarmAndUncached(first, writer, rules);
        copyProject(writer, reader);
        const moved = await build(reader, rules);
        expect(moved.built).toEqual(["resource.jsx"]);
        expectMapped(moved, fixed);
        await expectWarmAndUncached(moved, reader, rules);
        await expectColdEquality(moved, reader, rules);
      });
    }
  },
  {
    name: "automatic-map-uncertified-outcomes",
    description: "preserves inline, malformed, explicit JSON, absolute and symlink map behavior without inferring authority",
    async run() {
      await withProjects(async (writer, reader) => {
        const fixed = "https://example.invalid/fixed.tsx";
        const inline = Buffer.from(JSON.stringify(map(fixed))).toString("base64");
        writeProject(writer, {
          "inline.jsx": jsxSource() + `//# ${sourceMappingURL}data:application/json;base64,${inline}`,
          "inline-option.jsx": jsxSource() + `//# ${sourceMappingURL}data:application/json;base64,${inline}`,
          "json.jsx": jsxSource(),
          "malformed.jsx": jsxSource(),
          "malformed-inline.jsx": jsxSource() + `//# ${sourceMappingURL}data:broken`,
          "absolute.jsx": jsxSource() + `//# ${sourceMappingURL}${path.join(writer, "src", "fixed.map")}`,
          "symlink.jsx": jsxSource(),
          "dangling.jsx": jsxSource(),
          "control.jsx": jsxSource()
        });
        writeMap(writer, "fixed.map", fixed);
        fs.writeFileSync(path.join(writer, "src", "malformed.jsx.map"), "not a source map");
        fs.symlinkSync("fixed.map", path.join(writer, "src", "symlink.jsx.map"));
        fs.symlinkSync("absent.map", path.join(writer, "src", "dangling.jsx.map"));
        const rules = [
          swcRule(/^(?!.*(?:json|inline-option)\.jsx$).*\.jsx$/, autoMapOptions),
          swcRule(/[\\/]json\.jsx$/, { ...autoMapOptions, inputSourceMap: JSON.stringify(map(fixed)) }),
          swcRule(/inline-option\.jsx$/, { ...autoMapOptions, inputSourceMap: "inline" })
        ];
        const first = await build(writer, rules);
        expectMapped(first, fixed);
        // The existing snapshot policy cannot reuse a dangling-link observation, even
        // within one root. Assert that behavior explicitly instead of claiming authority.
        await expectWarmAndUncached(first, writer, rules, undefined, ["dangling.jsx"]);
        copyProject(writer, reader);
        const moved = await build(reader, rules);
        expect(moved.built).toEqual([
          "absolute.jsx", "dangling.jsx", "inline-option.jsx", "inline.jsx", "json.jsx",
          "malformed-inline.jsx", "malformed.jsx", "symlink.jsx"
        ]);
        expectMapped(moved, fixed);
        await expectWarmAndUncached(moved, reader, rules, undefined, ["dangling.jsx"]);
        await expectColdEquality(moved, reader, rules);
      });
    }
  },
  {
    name: "map-filesystem-observations",
    description: "uses the loader validation filesystem once and distinguishes permission failures from missing probes",
    async run() {
      await withProjects(async (writer, reader) => {
        writeProject(writer, {
          "unreadable.jsx": jsxSource(),
          "blocked.jsx": jsxSource() + `//# ${sourceMappingURL}blocked.map`,
          "virtual.jsx": jsxSource(),
          "control.jsx": jsxSource()
        });
        writeMap(writer, "unreadable.jsx.map", "must-not-read.tsx");
        writeMap(writer, "blocked.jsx.map", "fallback-after-permission.tsx");
        const reads = [];
        const denied = () => Object.assign(new Error("audited EACCES"), { code: "EACCES" });
        const configuration = {
          experiments: { useInputFileSystem: [/\.map$/] },
          configureCompiler(compiler) {
            const original = compiler.inputFileSystem;
            const replacement = Object.create(original);
            replacement.stat = (file, ...args) => {
              if (path.basename(file) === "blocked.map") return args.at(-1)(denied());
              if (path.basename(file) === "virtual.jsx.map") return original.stat(path.join(compiler.context, "src", "blocked.jsx.map"), ...args);
              return original.stat(file, ...args);
            };
            replacement.readFile = (file, ...args) => {
              reads.push(file);
              if (path.basename(file) === "unreadable.jsx.map") return args.at(-1)(denied());
              if (path.basename(file) === "virtual.jsx.map") return args.at(-1)(null, Buffer.from(JSON.stringify(map("virtual-filesystem.tsx"))));
              return original.readFile(file, ...args);
            };
            compiler.inputFileSystem = replacement;
          }
        };
        const rules = [swcRule(/\.jsx$/, autoMapOptions)];
        const uncached = await build(writer, rules, false, configuration);
        expectMapped(uncached, "virtual-filesystem.tsx");
        expectMapped(uncached, "fallback-after-permission.tsx");
        expect(uncached.map).not.toContain("must-not-read.tsx");
        expect(uncached.fileDependencies).toContain(path.join(writer, "src", "blocked.map"));
        expect(uncached.missingDependencies).not.toContain(path.join(writer, "src", "blocked.map"));
        // Each actual resolution performs a single read, even on a failed read/fallback.
        for (const name of ["unreadable.jsx.map", "blocked.jsx.map", "virtual.jsx.map"]) {
          expect(reads.filter(file => file === path.join(writer, "src", name))).toHaveLength(1);
        }
        const first = await build(writer, rules, undefined, configuration);
        expectEqualOutput(first, uncached);
        copyProject(writer, reader);
        const moved = await build(reader, rules, undefined, configuration);
        expect(moved.built).toEqual(["blocked.jsx", "unreadable.jsx", "virtual.jsx"]);
        expectEqualOutput(await build(reader, rules, false, configuration), moved);
        await expectColdEquality(moved, reader, rules, configuration);
      });
    }
  },
  {
    name: "decorator-option-boundary",
    description: "declines inferred or ECMAScript decorators and unaudited transform fields while allowing decorators:false",
    async run() {
      await withProjects(async (writer, reader) => {
        const variants = {
          "ecmascript.jsx": { ...swcOptions.jsc, parser: { syntax: "ecmascript", jsx: true, decorators: true } },
          "inferred.tsx": { ...swcOptions.jsc, parser: { decorators: true } },
          "metadata.tsx": { ...automaticOptions.jsc, transform: { ...automaticOptions.jsc.transform, decoratorMetadata: true } },
          "version.tsx": { ...automaticOptions.jsc, transform: { ...automaticOptions.jsc.transform, decoratorVersion: "2021-12" } },
          "legacy.tsx": { ...automaticOptions.jsc, transform: { ...automaticOptions.jsc.transform, legacyDecorator: true } },
          "false.jsx": { ...swcOptions.jsc, parser: { syntax: "ecmascript", jsx: true, decorators: false } }
        };
        writeProject(writer, Object.fromEntries(Object.keys(variants).map(name => [name, jsxSource()])));
        const rules = Object.entries(variants).map(([name, jsc]) => swcRule(new RegExp(`${name.replace('.', '\\.')}$`), { ...swcOptions, jsc }));
        const first = await build(writer, rules);
        await expectWarmAndUncached(first, writer, rules);
        copyProject(writer, reader);
        const moved = await build(reader, rules);
        expect(moved.built).toEqual(["ecmascript.jsx", "inferred.tsx", "legacy.tsx", "metadata.tsx", "version.tsx"]);
        await expectColdEquality(moved, reader, rules);
      });
    }
  }
];
