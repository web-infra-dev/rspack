import { createHash } from "node:crypto";
import { HtmlRspackPlugin, SubresourceIntegrityPlugin } from "@rspack/core";

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
	{
		name: "merge",
		description: "should merge final module integrity hashes into the page import map",
		options() {
			return {
				entry: { main: "./index.js", other: "./other.js" },
				output: {
					module: true,
					filename: "entries/[name].[contenthash:8].mjs",
					chunkFilename: "chunks/[name].[contenthash:8].mjs",
					publicPath: "https://cdn.example.test/assets/",
					crossOriginLoading: "anonymous"
				},
				plugins: [
					new HtmlRspackPlugin({
						chunks: ["main"],
						scriptLoading: "module",
						templateContent: `<!DOCTYPE html><html><head>
							<script type="importmap" nonce="test-nonce">{
								"imports": {"external": "/vendor/external.mjs"},
								"scopes": {"/vendor/": {"shared": "/vendor/shared.mjs"}},
								"integrity": {"/vendor/external.mjs": "sha384-external"}
							}</script>
							<link rel="modulepreload" href="/vendor/external.mjs">
						</head><body></body></html>`
					}),
					new HtmlRspackPlugin({
						chunks: ["other"],
						filename: "other.html",
						scriptLoading: "module"
					}),
					new SubresourceIntegrityPlugin()
				],
				optimization: {
					concatenateModules: false,
					splitChunks: {
						cacheGroups: {
							shared: { test: /shared\.js$/, name: "shared", chunks: "async", enforce: true }
						}
					}
				}
			};
		},
		check({ files }) {
			const html = files["/index.html"];
			const mapScripts = [...html.matchAll(/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/g)];
			expect(mapScripts).toHaveLength(1);
			expect(mapScripts[0][0]).toContain("test-nonce");
			expect(mapScripts[0].index).toBeLessThan(html.indexOf("modulepreload"));
			const map = JSON.parse(mapScripts[0][1]);
			expect(map.imports).toEqual({ external: "/vendor/external.mjs" });
			expect(map.scopes).toEqual({ "/vendor/": { shared: "/vendor/shared.mjs" } });
			expect(map.integrity["/vendor/external.mjs"]).toBe("sha384-external");
			const chunks = Object.keys(files).filter(file => file.startsWith("/chunks/"));
			expect(chunks).toHaveLength(2);
			for (const file of chunks) {
				const url = `https://cdn.example.test/assets${file}`;
				expect(map.integrity[url]).toBe(`sha384-${createHash("sha384").update(files[file]).digest("base64")}`);
			}
			expect(Object.keys(map.integrity).some(url => url.includes("/other."))).toBe(false);
			expect(files["/other.html"]).not.toContain("importmap");
		}
	},
	{
		name: "insert",
		description: "should insert an integrity map before the first module load",
		options() {
			return {
				entry: "./index.js",
				output: { module: true, crossOriginLoading: "anonymous" },
				plugins: [
					new HtmlRspackPlugin({
						filename: "pages/index.html",
						scriptLoading: "module",
						templateContent: '<!DOCTYPE html><html><head><link rel="modulepreload" href="/vendor/external.mjs" nonce="test-nonce"></head><body></body></html>'
					}),
					new SubresourceIntegrityPlugin()
				]
			};
		},
		check({ files }) {
			const html = files["/pages/index.html"];
			const mapScript = html.match(/<script\b[^>]*type=["']?importmap["']?[^>]*>([\s\S]*?)<\/script>/);
			expect(mapScript).toBeTruthy();
			expect(mapScript[0]).toContain("test-nonce");
			expect(mapScript.index).toBeLessThan(html.indexOf("modulepreload"));
			const chunk = Object.keys(files).find(file => file.endsWith(".mjs") && file !== "/main.mjs");
			expect(JSON.parse(mapScript[1]).integrity[`..${chunk}`]).toBe(
				`sha384-${createHash("sha384").update(files[chunk]).digest("base64")}`
			);
		}
	}
];
