const { RuntimeTemplate } = require("@rspack/core");

let runtimeTemplate;

class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			runtimeTemplate = compilation.runtimeTemplate;
		});
	}
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
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
					templateLiteral: false,
					nodePrefixForCoreModules: false
				}
			},
			plugins: [new MyPlugin()]
		};
	},
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.run(() => {
				resolve();
			});
		});
	},
	async check() {
		expect(runtimeTemplate).toBeInstanceOf(RuntimeTemplate);

		// the environment configured above, without any ES2015 feature
		expect(runtimeTemplate.supportsArrowFunction()).toBe(false);
		expect(runtimeTemplate.supportsDestructuring()).toBe(false);
		expect(runtimeTemplate.supportsForOf()).toBe(false);
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
				templateLiteral: true,
				nodePrefixForCoreModules: true
			}
		});
		expect(modern.returningFunction("a", "b")).toBe("(b) => (a)");
		expect(modern.basicFunction("a", "return a;")).toBe(
			"(a) => {\n\treturn a;\n}"
		);
		expect(modern.expressionFunction("a()")).toBe("() => (a())");
		expect(modern.emptyFunction()).toBe("x => {}");
		expect(modern.iife("", "a();")).toBe("(() => {\n\ta();\n})()");
		expect(modern.destructureArray(["a", "b"], "c")).toBe("var [a, b] = c;");
		expect(modern.destructureObject(["a", "b"], "c")).toBe("var {a, b} = c;");
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
};
