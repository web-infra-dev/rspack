/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
	description: "should load @swc/plugin-remove-console successfully and transform code using rspack inner swc api",
	async check(_, compiler, __) {
		let swc = compiler.rspack.experiments.swc;

		async function check_transform_api(transformApi) {
			let source = 'function main() { console.log("Hello Rspack") }; main();';
			let result = await transformApi(source, {
				filename: "index.js",
				minify: true,
				jsc: {
					parser: {
						syntax: "ecmascript",
						dynamicImport: true
					},
					target: "es5",
					experimental: {
						plugins: [[require.resolve("@swc/plugin-remove-console"), {}]],
					}
				}
			});

			expect(result.code).toMatchInlineSnapshot(`function main(){;};main();`);
		}

		await Promise.all([
			check_transform_api(swc.transform),
			check_transform_api(swc.transformSync)
		]);
	}
}
