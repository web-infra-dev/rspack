/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
	description: "should output sourcemaps when sourceMaps option is enabled in swc API",
	async check(_, compiler, __) {
		let swc = compiler.rspack.experiments.swc;

		async function check_transform_sourcemap(transformApi) {
			let source = 'function main() { console.log("Hello Rspack") }; main();';
			
			// Test with sourcemaps enabled
			let result = await transformApi(source, {
				filename: "index.js",
				sourceMaps: true,
				jsc: {
					parser: {
						syntax: "ecmascript"
					}
				}
			});

			expect(result.map).toBeDefined();
			expect(typeof result.map).toBe('string');
			
			// Verify sourcemap has required properties
			const sourceMap = JSON.parse(result.map);
			expect(sourceMap).toHaveProperty('version');
			expect(sourceMap).toHaveProperty('sources');
			expect(sourceMap).toHaveProperty('mappings');
		}

		await Promise.all([
			check_transform_sourcemap(swc.transform),
			check_transform_sourcemap(swc.transformSync)
		]);
	}
}
