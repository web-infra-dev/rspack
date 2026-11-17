import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

const require = createRequire(import.meta.url);

function project(root) {
	fs.mkdirSync(root, { recursive: true });
	fs.mkdirSync(path.join(root, ".git"));
	fs.writeFileSync(
		path.join(root, "package.json"),
		'{"name":"compilation-hash-order"}'
	);
	fs.writeFileSync(
		path.join(root, "index.js"),
		'console.log(__webpack_hash__); import "./style.css"; import(/* webpackChunkName: "lazy", webpackPrefetch: true */ "./lazy.js");'
	);
	fs.writeFileSync(
		path.join(root, "lazy.js"),
		'import "./lazy.css"; export const value = 1;'
	);
	fs.writeFileSync(path.join(root, "style.css"), ".app { color: red }");
	fs.writeFileSync(path.join(root, "lazy.css"), ".lazy { color: blue }");
	fs.writeFileSync(path.join(root, "dark.css"), ".dark { color: black }");
	fs.writeFileSync(path.join(root, "light.css"), ".light { color: white }");
}

async function build(
	root,
	{ reverse = false, cache = false, portableCache = { seed: false } } = {}
) {
	let builds = 0;
	const compiler = rspack({
		context: root,
		mode: "development",
		devtool: "source-map",
		incremental: false,
		// Entry insertion changes chunk allocation order without changing chunk identities.
		entry: Object.fromEntries(
			(reverse ? ["light", "dark", "app"] : ["app", "dark", "light"]).map(
				(name) => [name, name === "app" ? "./index.js" : `./${name}.css`]
			)
		),
		cache,
		experiments: { portableCache },
		output: {
			path: path.join(root, "dist"),
			clean: true,
			filename: "[name].[contenthash].js",
			chunkFilename: "[name].[contenthash].js",
			module: true,
			chunkFormat: "module",
			chunkLoading: "import"
		},
		optimization: {
			realContentHash: false,
			splitChunks: false,
			runtimeChunk: false
		},
		module: {
			rules: [
				{
					test: /\.css$/,
					use: [
						rspack.CssExtractRspackPlugin.loader,
						{
							loader: require.resolve("css-loader"),
							options: { sourceMap: false }
						}
					]
				}
			]
		},
		plugins: [
			new rspack.CssExtractRspackPlugin({
				filename: "[name].[contenthash].css"
			}),
			{
				apply(compiler) {
					compiler.hooks.thisCompilation.tap(
						"CompilationHashOrder",
						(compilation) => {
							compilation.hooks.buildModule.tap("CompilationHashOrder", () => {
								builds++;
							});
						}
					);
				}
			}
		],
		infrastructureLogging: { level: "error" }
	});
	try {
		const stats = await runCompiler(compiler);
		const { errors, warnings } = stats.toJson({
			all: false,
			errors: true,
			warnings: true
		});
		expect(errors).toEqual([]);
		expect(warnings).toEqual([]);
		const dist = path.join(root, "dist");
		const assets = Object.fromEntries(
			fs
				.readdirSync(dist)
				.sort()
				.map((name) => [name, fs.readFileSync(path.join(dist, name))])
		);
		expect(Object.keys(assets).some((name) => name.endsWith(".js.map"))).toBe(
			true
		);
		expect(Object.keys(assets).some((name) => name.endsWith(".css"))).toBe(
			true
		);
		return { assets, builds, hash: stats.compilation.fullHash };
	} finally {
		await closeCompiler(compiler);
	}
}

export default [
	{
		name: "chunk-allocation-order",
		description: "hashes chunks by stable IDs regardless of allocation order",
		async run() {
			const root = fs.realpathSync.native(
				fs.mkdtempSync(path.join(os.tmpdir(), "rspack-chunk-order-"))
			);
			try {
				project(path.join(root, "app"));
				const first = await build(path.join(root, "app"));
				const reversed = await build(path.join(root, "app"), { reverse: true });
				expect(reversed.hash).toBe(first.hash);
				expect(reversed.assets).toEqual(first.assets);
			} finally {
				fs.rmSync(root, { recursive: true, force: true });
			}
		}
	},
	{
		name: "persistent-extract-css",
		description:
			"keeps raw JS, maps and extracted CSS equal across cold, warm and seeded compilers",
		async run() {
			const root = fs.realpathSync.native(
				fs.mkdtempSync(path.join(os.tmpdir(), "rspack-hash-cache-"))
			);
			try {
				const donor = path.join(root, "donor"),
					reader = path.join(root, "reader");
				project(donor);
				project(reader);
				const cache = (location) => ({
					type: "persistent",
					portable: true,
					storage: { type: "filesystem", location }
				});
				const donorCache = path.join(donor, "cache"),
					readerCache = path.join(reader, "cache");
				const first = await build(donor, { cache: cache(donorCache) });
				expect(first.builds).toBeGreaterThan(0);
				const warm = await build(donor, { cache: cache(donorCache) });
				expect(warm.builds).toBe(0);
				expect(warm.assets).toEqual(first.assets);
				const seeded = await build(reader, {
					cache: cache(readerCache),
					portableCache: { seedFrom: donorCache }
				});
				const readerWarm = await build(reader, { cache: cache(readerCache) });
				expect(readerWarm.builds).toBe(0);
				const cold = await build(reader, {
					cache: cache(path.join(reader, "cold-cache"))
				});
				expect(cold.builds).toBeGreaterThan(0);
				expect(seeded.assets).toEqual(cold.assets);
				expect(readerWarm.assets).toEqual(cold.assets);
			} finally {
				fs.rmSync(root, { recursive: true, force: true });
			}
		}
	}
];
