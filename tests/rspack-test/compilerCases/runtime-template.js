const { RuntimeTemplate } = require("@rspack/core");

let runtimeTemplate;
let neutralPlatform;

class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			runtimeTemplate = compilation.runtimeTemplate;
		});
	}
}

class PlatformPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			neutralPlatform = compilation.runtimeTemplate.isNeutralPlatform();
		});
	}
}

const build = async (_, compiler) => {
	await new Promise(resolve => {
		compiler.run(() => {
			resolve();
		});
	});
};

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	{
		description: "should expose compilation.runtimeTemplate",
		options(context) {
			return {
				context: context.getSource(),
				entry: "./d",
				output: {
					environment: {
						arrowFunction: false,
						destructuring: false,
						forOf: false,
						spread: false,
						templateLiteral: false,
						nodePrefixForCoreModules: false
					}
				},
				plugins: [new MyPlugin()]
			};
		},
		build,
		async check() {
			expect(runtimeTemplate).toBeInstanceOf(RuntimeTemplate);

			// `target` defaults to `web`, so the platform is not neutral
			expect(runtimeTemplate.isNeutralPlatform()).toBe(false);

			// the environment configured above, without any ES2015 feature
			expect(runtimeTemplate.supportsArrowFunction()).toBe(false);
			expect(runtimeTemplate.supportsDestructuring()).toBe(false);
			expect(runtimeTemplate.supportsForOf()).toBe(false);
			expect(runtimeTemplate.supportsSpread()).toBe(false);
			expect(runtimeTemplate.supportTemplateLiteral()).toBe(false);
			expect(runtimeTemplate.returningFunction("a", "b")).toBe(
				"function(b) { return a; }"
			);
			expect(runtimeTemplate.basicFunction("a", "return a;")).toBe(
				"function(a) {\n\treturn a;\n}"
			);
			expect(runtimeTemplate.expressionFunction("a()")).toBe(
				"function() { a(); }"
			);
			expect(runtimeTemplate.emptyFunction()).toBe("function() {}");
			expect(runtimeTemplate.iife("", "a();")).toBe(
				"(function() {\n\ta();\n})()"
			);
			expect(runtimeTemplate.destructureArray(["a", "b"], "c")).toBe(
				"var a = c[0];\nvar b = c[1];"
			);
			expect(runtimeTemplate.destructureObject(["a", "b"], "c")).toBe(
				"var a = c.a;\nvar b = c.b;"
			);
			expect(runtimeTemplate.forEach("a", "b", "c(a);")).toBe(
				"b.forEach(function(a) {\n\tc(a);\n});"
			);
			expect(runtimeTemplate.concatenation("a", { expr: "b" }, "c")).toBe(
				'"a" + b + "c"'
			);
			expect(runtimeTemplate.renderNodePrefixForCoreModule("fs")).toBe('"fs"');
			expect(
				runtimeTemplate.missingModuleStatement({ request: "./missing" })
			).toBe(
				`Object(function webpackMissingModule() { var e = new Error("Cannot find module './missing'"); e.code = 'MODULE_NOT_FOUND'; throw e; }());\n`
			);

			// the same helpers with every ES2015 feature enabled
			const modern = new RuntimeTemplate(runtimeTemplate.compilation, {
				...runtimeTemplate.outputOptions,
				environment: {
					...runtimeTemplate.outputOptions.environment,
					arrowFunction: true,
					destructuring: true,
					forOf: true,
					spread: true,
					templateLiteral: true,
					nodePrefixForCoreModules: true
				}
			});
			expect(modern.supportsSpread()).toBe(true);
			expect(modern.returningFunction("a", "b")).toBe("(b) => (a)");
			expect(modern.basicFunction("a", "return a;")).toBe(
				"(a) => {\n\treturn a;\n}"
			);
			expect(modern.expressionFunction("a()")).toBe("() => (a())");
			expect(modern.emptyFunction()).toBe("x => {}");
			expect(modern.iife("", "a();")).toBe("(() => {\n\ta();\n})()");
			expect(modern.destructureArray(["a", "b"], "c")).toBe("var [a, b] = c;");
			expect(modern.destructureObject(["a", "b"], "c")).toBe(
				"var {a, b} = c;"
			);
			expect(modern.forEach("a", "b", "c(a);")).toBe(
				"for(const a of b) {\n\tc(a);\n}"
			);
			expect(
				modern.concatenation(
					"x",
					{ expr: "a" },
					"x",
					{ expr: "b" },
					"x",
					{ expr: "c" },
					"x"
				)
			).toBe("`x${a}x${b}x${c}x`");
			expect(modern.renderNodePrefixForCoreModule("fs")).toBe('"node:fs"');
		}
	},
	{
		description: "should not report a web worker as a neutral platform",
		options(context) {
			return {
				context: context.getSource(),
				entry: "./d",
				target: "webworker",
				plugins: [new PlatformPlugin()]
			};
		},
		build,
		async check() {
			expect(neutralPlatform).toBe(false);
		}
	},
	{
		description: "should report a platform agnostic target as neutral",
		options(context) {
			return {
				context: context.getSource(),
				entry: "./d",
				// without a target there is no platform to infer a chunk format from
				target: false,
				output: {
					chunkFormat: "array-push",
					chunkLoading: false
				},
				plugins: [new PlatformPlugin()]
			};
		},
		build,
		async check() {
			expect(neutralPlatform).toBe(true);
		}
	}
];
