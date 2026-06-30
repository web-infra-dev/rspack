const fs = require("fs");
const path = require("path");

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
}, {
	description: "should preserve input sourcemap ignoreList in swc API",
	async check({ compiler }) {
		let swc = compiler.rspack.experiments.swc;

		async function check_transform_input_ignore_list(transformApi) {
			let source = 'console.log("Hello Rspack");';
			let inputSourceMap = JSON.stringify({
				version: 3,
				file: "index.js",
				sources: ["vendor.js"],
				sourcesContent: [source],
				names: [],
				mappings: "AAAA",
				ignoreList: [0]
			});

			let result = await transformApi(source, {
				filename: "index.js",
				sourceMaps: true,
				inputSourceMap,
				jsc: {
					parser: {
						syntax: "ecmascript"
					}
				}
			});

			const sourceMap = JSON.parse(result.map);
			const ignoredIndex = sourceMap.sources.indexOf("vendor.js");
			expect(ignoredIndex).not.toBe(-1);
			expect(sourceMap.ignoreList).toContain(ignoredIndex);
		}

		await Promise.all([
			check_transform_input_ignore_list(swc.transform),
			check_transform_input_ignore_list(swc.transformSync)
		]);
	}
}, {
	description: "should accept input sourcemaps with garbage headers in swc API",
	async check({ compiler, context }) {
		let swc = compiler.rspack.experiments.swc;

		async function check_transform_input_garbage_header(transformApi) {
			let source = 'console.log("Hello Rspack");';
			let inputSourceMap = JSON.stringify({
				version: 3,
				file: "index.js",
				sources: ["original.js"],
				sourcesContent: [source],
				names: [],
				mappings: "AAAA"
			});
			let inputSourceMapWithHeader = `)]}'\n${inputSourceMap}`;

			let userProvidedResult = await transformApi(source, {
				filename: "index.js",
				sourceMaps: true,
				inputSourceMap: inputSourceMapWithHeader,
				jsc: {
					parser: {
						syntax: "ecmascript"
					}
				}
			});

			expect(JSON.parse(userProvidedResult.map).sources).toContain("original.js");

			let inlineResult = await transformApi(
				`${source}\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,${Buffer.from(inputSourceMapWithHeader).toString("base64")}`,
				{
					filename: "index.js",
					sourceMaps: true,
					inputSourceMap: true,
					jsc: {
						parser: {
							syntax: "ecmascript"
						}
					}
				}
			);

			expect(JSON.parse(inlineResult.map).sources).toContain("original.js");

			const filename = context.getDist("source-map-garbage-header.js");
			const mapFilename = `${filename}.map`;
			fs.mkdirSync(path.dirname(mapFilename), { recursive: true });
			fs.writeFileSync(mapFilename, inputSourceMapWithHeader, "utf-8");

			let fileResult = await transformApi(
				`${source}\n//# sourceMappingURL=${path.basename(mapFilename)}`,
				{
					filename,
					sourceMaps: true,
					inputSourceMap: true,
					jsc: {
						parser: {
							syntax: "ecmascript"
						}
					}
				}
			);

			expect(JSON.parse(fileResult.map).sources).toContain("original.js");
		}

		await Promise.all([
			check_transform_input_garbage_header(swc.transform),
			check_transform_input_garbage_header(swc.transformSync)
		]);
	}
}]
