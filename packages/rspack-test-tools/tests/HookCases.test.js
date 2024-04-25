const path = require("path");
const fs = require("fs");
const { Compiler, Compilation } = require("@rspack/core");
const { getSerializers } = require("jest-snapshot");
const pathSerializer = require("jest-serializer-path");
const prettyFormat = require("pretty-format");
const createLazyTestEnv = require("../src/helper/legacy/createLazyTestEnv");
const { Source } = require("webpack-sources");
const normalizePaths = pathSerializer.normalizePaths;
const srcDir = path.resolve(__dirname, "./fixtures");
const distDir = path.resolve(__dirname, "./js/HookTestCases");
const caseDir = path.resolve(__dirname, "./hookCases");
const {
	HookTaskProcessor,
	TestContext,
	ECompilerType,
	isValidCaseDirectory,
	isDirectory,
	BasicRunnerFactory
} = require("..");

const sourceSerializer = {
	test(val) {
		return val instanceof Source;
	},
	print(val) {
		return val.source();
	}
};

const internalSerializer = {
	test(val) {
		return val instanceof Compiler || val instanceof Compilation;
	},
	print(val) {
		return JSON.stringify(`${val.constructor.name}(internal ignored)`);
	}
};

const testPathSerializer = {
	test(val) {
		return typeof val === "string";
	},
	print(val) {
		return JSON.stringify(
			normalizePaths(
				val
					.replaceAll(srcDir, "<HOOK_SRC_DIR>")
					.replaceAll(distDir, "<HOOK_DIST_DIR>")
			)
		);
	}
};

const escapeRegex = true;
const printFunctionName = false;
const normalizeNewlines = string => string.replace(/\r\n|\r/g, "\n");
const serialize = (val, indent = 2, formatOverrides = {}) =>
	normalizeNewlines(
		prettyFormat.format(val, {
			escapeRegex,
			indent,
			plugins: [
				...getSerializers(),
				sourceSerializer,
				internalSerializer,
				testPathSerializer
			],
			printFunctionName,
			...formatOverrides
		})
	);

class HookCasesContext extends TestContext {
	constructor(name, testName, options) {
		super(options);
		this.snapshots = {};
		this.snapshotsList = [];
		this.name = name;
		this.testName = testName;
		this.promises = [];
		this.snapped = this.snapped.bind(this);
		this.count = 0;
	}

	/**
	 * Snapshot function arguments and return value.
	 * Generated snapshot is located in the same directory with the test source.
	 * @example
	 * compiler.hooks.compilation("name", context.snapped((...args) => { ... }))
	 */
	snapped(cb, prefix = "") {
		let context = this;
		return function SNAPPED_HOOK(...args) {
			let group = prefix ? prefix : context.count++;
			context._addSnapshot(args, "input", group);
			let output = cb.apply(this, args);
			if (output && typeof output.then === "function") {
				let resolve;
				context.promises.push(new Promise(r => (resolve = r)));
				return output
					.then(o => {
						context._addSnapshot(o, "output (Promise resolved)", group);
						return o;
					})
					.catch(o => {
						context._addSnapshot(o, "output (Promise rejected)", group);
						return o;
					})
					.finally(resolve);
			}
			context._addSnapshot(output, "output", group);
			return output;
		};
	}

	/**
	 * @internal
	 */
	_addSnapshot(content, name, group) {
		content = Buffer.isBuffer(content)
			? content
			: serialize(content, undefined, {
				escapeString: true,
				printBasicPrototype: true
			}).replace(/\r\n/g, "\n");
		(this.snapshots[group] = this.snapshots[group] || []).push([content, name]);
		if (!this.snapshotsList.includes(group)) {
			this.snapshotsList.push(group);
		}
	}

	/**
	 * @internal
	 */
	async collectSnapshots(
		options = {
			diff: {}
		}
	) {
		await Promise.allSettled(this.promises);
		if (!this.snapshotsList.length) return;

		let snapshots = this.snapshotsList.reduce((acc, group, index) => {
			let block = this.snapshots[group || index].reduce(
				(acc, [content, name]) => {
					name = `## ${name || `test: ${index}`}\n\n`;
					let block = "```javascript\n" + content + "\n```\n";
					return (acc += name + block + "\n");
				},
				""
			);
			group = Number.isInteger(group) ? `Group: ${index}` : group;
			group = `# ${group}\n\n`;
			return (acc += group + block);
		}, "");

		expect(snapshots).toMatchFileSnapshot(
			path.join(path.dirname(this.name), "hooks.snap.txt"),
			options
		);
	}
}

describe("Hook", () => {
	const categories = fs
		.readdirSync(caseDir)
		.filter(isValidCaseDirectory)
		.filter(folder => isDirectory(path.join(caseDir, folder)))
		.map(cat => {
			return {
				name: cat,
				tests: fs
					.readdirSync(path.join(caseDir, cat))
					.map(i => {
						if (isDirectory(path.join(caseDir, cat, i))) {
							return i;
						}
					})
					.filter(Boolean)
					.sort()
			};
		});

	for (let cat of categories) {
		describe(cat.name, () => {
			for (let name of cat.tests) {
				async function run(_name, testName, processor) {
					const context = new HookCasesContext(_name, testName, {
						src: srcDir,
						dist: path.join(distDir, cat.name, name),
						runnerFactory: BasicRunnerFactory
					});
					try {
						await processor.before(context);
						await processor.config(context);
						await processor.compiler(context);
						await processor.build(context);
						await processor.run(env, context);
					} catch (e) {
						throw e;
					} finally {
						await context.collectSnapshots();
						await processor.check(null, context);
						await processor.after(context);
					}
				}

				let file = path.join(caseDir, cat.name, name, "test.js");
				const caseConfig = require(file);
				it(caseConfig.description, async () => {
					await run(
						file,
						path.basename(name.slice(0, name.indexOf(path.extname(name)))),
						new HookTaskProcessor({
							name: file,
							compilerType: ECompilerType.Rspack,
							findBundle: function (i, options) {
								return ["main.js"];
							},
							snapshot: path.join(caseDir, cat.name, name, "output.snap.txt"),
							...caseConfig
						})
					);
				});
				const env = createLazyTestEnv(1000);
			}
		});
	}
});
