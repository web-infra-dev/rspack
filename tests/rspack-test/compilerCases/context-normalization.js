const path = require("node:path");
const rspack = require("@rspack/core");

const close = compiler =>
	new Promise((resolve, reject) => {
		compiler.close(error => {
			if (error) {
				reject(error);
				return;
			}
			resolve();
		});
	});

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	{
		description:
			"should normalize a non-empty context to native separators (#14446)",
		options(context) {
			return {
				context: context.getSource(),
				output: {
					path: context.getDist()
				}
			};
		},
		async check({ context }) {
			const src = context.getSource();

			const unnormalized = `${src}${path.sep}sub${path.sep}..`;
			const compiler = rspack({
				entry: context.getSource("a.js"),
				context: unnormalized,
				output: { path: context.getDist() }
			});
			expect(compiler.options.context).toBe(path.resolve(unnormalized));
			expect(compiler.options.context).toBe(src);
			await close(compiler);

			if (process.platform === "win32") {
				const forwardSlash = src.replace(/\\/g, "/");
				const winCompiler = rspack({
					entry: context.getSource("a.js"),
					context: forwardSlash,
					output: { path: context.getDist() }
				});
				expect(winCompiler.options.context).toBe(src);
				expect(winCompiler.options.context).not.toContain("/");
				await close(winCompiler);
			}
		}
	}
];
