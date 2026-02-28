/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [{
	description: "should load @swc/plugin-remove-console successfully and transform code using rspack inner swc api",
	async check({ compiler }) {
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
}, {
	description: "should output sourcemaps when sourceMaps option is enabled in swc API",
	async check({ compiler }) {
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
}]
