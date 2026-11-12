import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import auditedCases from "./audited-cases.mjs";
import {
	build,
	copyProject,
	expectColdEquality,
	jsxSource,
	swcOptions,
	swcRule,
	withProjects,
	writeProject
} from "./helpers.mjs";

function expectRealPaths(result, root) {
	for (const name of Object.keys(result.sources)) {
		const value = result.values.find(
			(entry) => entry.value.fileName === path.join(root, "src", name)
		);
		expect(value).toBeDefined();
		expect(value.value.type).toBe("section");
		expect(value.value.line).toBeGreaterThan(0);
		expect(value.value.column).toBeGreaterThan(0);
		expect(result.sources[name]).not.toContain("[portable-root]");
	}
	for (const text of [result.output, result.map]) {
		expect(text).not.toContain("[portable-root]");
	}
}

// The raw bytes contain no resource path, but SWC's parsed string value does.
function escapeAll(value) {
	let escaped = "";
	for (let i = 0; i < value.length; i++) {
		escaped += `\\u${value.charCodeAt(i).toString(16).padStart(4, "0")}`;
	}
	return escaped;
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig[]} */
export default [
	...[false, true].map((refresh) => ({
		name: refresh ? "jsx-refresh-runtime" : "jsx-development",
		description:
			"reuses opt-in SWC JSX and TSX output with relocated dev filenames, maps and hashes equal to cold builds",
		async run() {
			await withProjects(async (writer, reader) => {
				writeProject(writer, {
					"component.jsx": jsxSource(),
					"typed.tsx": jsxSource("const count: number = 7;", "{count}")
				});
				const rules = [swcRule(/\.(jsx|tsx)$/, swcOptions, refresh)];
				const first = await build(writer, rules);
				expect(first.built).toEqual(["component.jsx", "typed.tsx"]);
				expectRealPaths(first, writer);
				const uncached = await build(writer, rules, false);
				expect(uncached.output).toBe(first.output);
				expect(uncached.map).toBe(first.map);
				expect(uncached.file).toBe(first.file);
				copyProject(writer, reader);
				const moved = await build(reader, rules);
				expect(moved.built).toEqual([]);
				expectRealPaths(moved, reader);
				expect(moved.values.map((entry) => entry.value.title)).toEqual([
					"portable", 7
				]);
				expect(moved.file).not.toBe(first.file);
				await expectColdEquality(moved, reader, rules);
			});
		}
	})),
	{
		name: "authored-enum-path",
		description:
			"keeps an authored TypeScript enum member that folds to the resource path fixed when the checkout moves",
		async run() {
			await withProjects(async (writer, reader) => {
				const resource = path.join(writer, "src", "enum-path.tsx");
				const split = resource.length - "enum-path.tsx".length;
				// No input literal equals the resource path; the enum transform folds one.
				writeProject(writer, {
					"enum-path.tsx": [
						`enum Resource { Path = ${JSON.stringify(resource.slice(0, split))} + ${JSON.stringify(resource.slice(split))} }`,
						"export const value = Resource.Path;",
						""
					].join("\n")
				});
				const rules = [swcRule(/\.tsx$/)];
				const first = await build(writer, rules);
				expect(first.built).toEqual(["enum-path.tsx"]);
				expect(first.values[0].value).toBe(resource);
				copyProject(writer, reader);
				const moved = await build(reader, rules);
				// An authored constant keeps its value; the module is bound to the writer root.
				expect(moved.values[0].value).toBe(resource);
				expect(moved.built).toEqual(["enum-path.tsx"]);
				await expectColdEquality(moved, reader, rules);
			});
		}
	},
	{
		name: "authored-resource-paths",
		description:
			"binds authored cooked resource paths while preserving their fixed values and relocating generated JSX filenames",
		async run() {
			await withProjects(async (writer, reader) => {
				const file = (name) => path.join(writer, "src", name);
				const names = [
					"literal.jsx", "escaped.jsx", "template.jsx", "attribute.jsx", "text.jsx"
				];
				const entityPath = (value) => [...value]
					.map((char) => `&#${char.codePointAt(0)};`)
					.join("");
				writeProject(writer, {
					"literal.jsx": jsxSource(
						`export const fixed = ${JSON.stringify(file(names[0]))};`
					),
					"escaped.jsx": jsxSource(
						`export const fixed = "${escapeAll(file(names[1]))}";`
					),
					"template.jsx": jsxSource(
						`export const fixed = \`${escapeAll(file(names[2]))}\`;`
					),
					"attribute.jsx": jsxSource("", `"${entityPath(file(names[3]))}"`),
					"text.jsx": jsxSource("", '"portable"', entityPath(file(names[4]))),
					"control.jsx": jsxSource()
				});
				const rules = [swcRule(/\.jsx$/)];
				const first = await build(writer, rules);
				expectRealPaths(first, writer);
				copyProject(writer, reader);
				const moved = await build(reader, rules);
				expect(moved.built).toEqual([...names].sort());
				expectRealPaths(moved, reader);
				for (let index = 0; index < 3; index++) {
					expect(moved.values[index].fixed).toBe(file(names[index]));
				}
				expect(moved.values[3].value.title).toBe(file(names[3]));
				expect(moved.values[4].value.children).toEqual([file(names[4])]);
				await expectColdEquality(moved, reader, rules);
			});
		}
	},
	...[
		{ target: "es5", charset: "utf8", escaped: "l\\xf6nger-读者-\\uD83D\\uDE00" },
		{
			target: "es2020",
			charset: "ascii",
			escaped: "l\\xf6nger-\\u8BFB\\u8005-\\u{1F600}"
		}
	].flatMap(({ target, charset, escaped }) => [true, false].map((portableOutput) => ({
		name: `escaped-writer-root-${target}-portable-output-${portableOutput}`,
		description:
			"rebuilds output that spells a non-ASCII writer root in ASCII escapes instead of reusing its stale path",
		async run() {
			await withProjects(async (reader, writer) => {
				writeProject(writer, { "component.jsx": jsxSource() });
				const rules = [swcRule(/\.jsx$/, {
					...swcOptions,
					rspackExperiments: { portableOutput },
					jsc: { ...swcOptions.jsc, target, output: { charset } }
				})];
				// A source map would hold the raw root and bind the entry for an unrelated reason.
				const configuration = { devtool: false };
				const first = await build(writer, rules, undefined, configuration);
				expect(first.built).toEqual(["component.jsx"]);
				expect(first.sources["component.jsx"]).toContain(escaped);
				expect(first.sources["component.jsx"]).not.toContain(writer);
				copyProject(writer, reader);
				const moved = await build(reader, rules, undefined, configuration);
				expect(moved.built).toEqual(["component.jsx"]);
				expect(moved.values[0].value.fileName).toBe(
					path.join(reader, "src", "component.jsx")
				);
				await expectColdEquality(moved, reader, rules, configuration);
			});
		}
	}))),
	...["es5", "es2020"].map((target) => ({
		name: `non-ascii-root-spelling-${target}`,
		description:
			"writes a JSX filename below a non-ASCII root with the bytes SWC writes without portable output",
		async run() {
			await withProjects(async (writer, reader) => {
				writeProject(writer, { "component.jsx": jsxSource() });
				const rule = (portableOutput) => swcRule(/\.jsx$/, {
					...swcOptions,
					rspackExperiments: { portableOutput },
					jsc: { ...swcOptions.jsc, target }
				});
				const rules = [rule(true)];
				await build(writer, rules);
				copyProject(writer, reader);
				const moved = await build(reader, rules);
				expect(moved.built).toEqual([]);
				await expectColdEquality(moved, reader, rules);
				const swc = await build(reader, [rule(false)], false);
				// SWC escapes part of this root for ES5, so the two targets write different bytes.
				expect(swc.sources["component.jsx"]).toContain(
					target === "es5" ? "l\\xf6nger-读者-\\uD83D\\uDE00" : "lönger-读者-😀"
				);
				expect(moved.sources).toEqual(swc.sources);
				// Only the hashed file name differs, in the map and in the comment that links it.
				const unnamed = (result) => [
					result.output.replaceAll(result.file, ""),
					result.map.replaceAll(result.file, "")
				];
				expect(unnamed(moved)).toEqual(unnamed(swc));
			});
		}
	})),
	{
		name: "excluded-options",
		description:
			"reuses default and explicit-target SWC configurations and keeps excluded ones and uncertified loader chains bound while compiling them successfully",
		async run() {
			await withProjects(async (writer, reader) => {
				writeProject(writer, {
					"chain.jsx": jsxSource(),
					"default.jsx": jsxSource(),
					"env-query.jsx": jsxSource(),
					"unknown.jsx": jsxSource(),
					"plugin.jsx": jsxSource(
						'console.log("removed by the WASM plugin");'
					),
					"env.jsx": jsxSource(
						"export const transformed = ({ count: 4 })?.count;"
					),
					"refresh.jsx": jsxSource(
						"export function Component() { return null; }"
					),
					"input-map.jsx": jsxSource() + "//# sourceMappingURL=input-map.jsx.map",
					"control.jsx": jsxSource()
				});
				fs.writeFileSync(
					path.join(writer, "plain-loader.cjs"),
					"module.exports = function (source) { return source; };"
				);
				fs.writeFileSync(
					path.join(writer, "src", "input-map.jsx.map"),
					JSON.stringify({
						version: 3,
						sources: ["authored.tsx"],
						sourcesContent: ["export const authored = 1;"],
						names: [],
						mappings: "AAAA"
					})
				);
				const chain = swcRule(/chain\.jsx$/);
				chain.use.unshift("./plain-loader.cjs");
				const rules = [
					chain,
					swcRule(/default\.jsx$/, { ...swcOptions, rspackExperiments: {} }),
					swcRule(/unknown\.jsx$/, { ...swcOptions, futurePortableOption: true }),
					swcRule(/plugin\.jsx$/, {
						...swcOptions,
						jsc: {
							...swcOptions.jsc,
							experimental: {
								plugins: [[
									fileURLToPath(import.meta.resolve("@swc/plugin-remove-console")),
									{}
								]]
							}
						}
					}),
					swcRule(/env\.jsx$/, {
						...swcOptions,
						jsc: { transform: swcOptions.jsc.transform },
						env: { targets: { chrome: "60" } }
					}),
					swcRule(/env-query\.jsx$/, {
						...swcOptions,
						jsc: { transform: swcOptions.jsc.transform },
						env: { targets: "chrome 60" }
					}),
					swcRule(
						/refresh\.jsx$/,
						{
							...swcOptions,
							jsc: {
								...swcOptions.jsc,
								transform: {
									react: { ...swcOptions.jsc.transform.react, refresh: true }
								}
							}
						},
						true
					),
					swcRule(/input-map\.jsx$/, { ...swcOptions, inputSourceMap: true }),
					swcRule(/control\.jsx$/)
				];
				const first = await build(writer, rules);
				expectRealPaths(first, writer);
				expect(first.sources["plugin.jsx"]).not.toContain(
					"removed by the WASM plugin"
				);
				expect(first.sources["env.jsx"]).not.toContain("?.");
				expect(
					first.values.find((entry) => "transformed" in entry).transformed
				).toBe(4);
				const map = JSON.parse(first.map);
				expect(
					map.sources.some((source) => source.endsWith("/authored.tsx"))
				).toBe(true);
				copyProject(writer, reader);
				const moved = await build(reader, rules);
				expect(moved.built).toEqual([
					"chain.jsx", "env-query.jsx", "input-map.jsx", "plugin.jsx",
					"unknown.jsx"
				]);
				expectRealPaths(moved, reader);
				await expectColdEquality(moved, reader, rules);
			});
		}
	},
	...auditedCases
];
