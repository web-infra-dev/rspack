import path from "node:path";
import { promisify } from "node:util";
import { runInNewContext } from "node:vm";

const plugin = {
	apply(compiler) {
		compiler.hooks.thisCompilation.tap("PublicPathPlugin", compilation => {
			const { RuntimeGlobals, RuntimeModule } = compiler.rspack;
			compilation.hooks.additionalTreeRuntimeRequirements.tap(
				"PublicPathPlugin",
				(_chunk, requirements) => requirements.add(RuntimeGlobals.publicPath)
			);
			compilation.hooks.runtimeRequirementInTree
				.for(RuntimeGlobals.publicPath)
				.tap("PublicPathPlugin", chunk => {
					const mod = new RuntimeModule(
						"plugin public path",
						RuntimeModule.STAGE_TRIGGER
					);
					mod.generate = () =>
						'__webpack_require__.p += "plugin/"; globalThis.pluginPath = __webpack_require__.p;';
					compilation.addRuntimeModule(chunk, mod);
				});
		});
	}
};

/** @type {import('@rspack/test-tools').TMultiCompilerCaseConfig} */
export default {
	description: "a shared plugin uses each compiler's runtime globals",
	options(context) {
		return ["webpack", "rspack"].map(runtimeMode => ({
			name: runtimeMode,
			mode: "development",
			target: "node",
			devtool: false,
			entry: "data:text/javascript,export default 1",
			experiments: { runtimeMode },
			output: {
				path: context.getDist(),
				filename: runtimeMode + ".js",
				publicPath: "/" + runtimeMode + "/"
			},
			plugins: [plugin]
		}));
	},
	async build(context, compiler) {
		try {
			const [first, second] = compiler.compilers;
			expect(first.rspack.RuntimeGlobals.publicPath).not.toBe(
				second.rspack.RuntimeGlobals.publicPath
			);
			const stats = await promisify(compiler.run.bind(compiler))();
			expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
			for (const child of compiler.compilers) {
				const file = path.join(context.getDist(), child.name + ".js");
				const scope = {};
				runInNewContext(child.outputFileSystem.readFileSync(file, "utf8"), scope);
				expect(scope.pluginPath).toBe("/" + child.name + "/plugin/");
			}
		} finally {
			await promisify(compiler.close.bind(compiler))();
		}
	}
};
