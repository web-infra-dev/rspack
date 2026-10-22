import { rspack } from "@rspack/core";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

function writeFixture(projectRoot, value) {
	fs.mkdirSync(path.join(projectRoot, "src"), { recursive: true });
	fs.writeFileSync(
		path.join(projectRoot, "src", "index.js"),
		'import("./value.js").then(({ default: value }) => console.log(value));\n'
	);
	fs.writeFileSync(
		path.join(projectRoot, "src", "value.js"),
		`export default ${JSON.stringify(value)};\n`
	);
}

function cacheDirectory(projectRoot) {
	return path.join(projectRoot, "node_modules", ".cache", "rspack");
}

async function build(projectRoot, { newCache, portable }) {
	const compiler = rspack({
		context: projectRoot,
		mode: "development",
		devtool: "source-map",
		target: "node",
		entry: "./src/index.js",
		experiments: { newCache },
		cache: { type: "persistent", portable },
		infrastructureLogging: { level: "none" },
		output: {
			path: path.join(projectRoot, "dist"),
			filename: "main.js",
			devtoolModuleFilenameTemplate: "[absolute-resource-path]"
		}
	});
	try {
		await new Promise((resolve, reject) => {
			compiler.run((error, stats) => {
				if (error) return reject(error);
				if (stats.hasErrors()) {
					return reject(
						new Error(stats.toString({ all: false, errors: true, errorDetails: true }))
					);
				}
				resolve();
			});
		});
	} finally {
		await new Promise((resolve, reject) =>
			compiler.close(error => (error ? reject(error) : resolve()))
		);
	}
	const dist = path.join(projectRoot, "dist");
	const files = fs.readdirSync(dist);
	return {
		code: files
			.filter(file => file.endsWith(".js"))
			.map(file => fs.readFileSync(path.join(dist, file), "utf8"))
			.join("\n"),
		sources: files
			.filter(file => file.endsWith(".map"))
			.flatMap(file => JSON.parse(fs.readFileSync(path.join(dist, file), "utf8")).sources)
			.filter(source => path.isAbsolute(source))
			.sort()
	};
}

function createCase({ newCache, portable }) {
	const backend = newCache ? "new" : "legacy";
	return {
		description: `should build from its own sources when a ${backend} persistent cache (portable: ${portable}) comes from another root`,
		async run() {
			const tempRoot = fs.realpathSync(
				fs.mkdtempSync(path.join(os.tmpdir(), "rspack-cache-other-root-"))
			);
			try {
				const options = { newCache, portable };
				const rootA = path.join(tempRoot, "a");
				const rootB = path.join(tempRoot, "b");
				writeFixture(rootA, "value-from-root-a");
				writeFixture(rootB, "value-from-root-b");

				const sourcesIn = projectRoot =>
					[
						path.join(projectRoot, "src", "index.js"),
						path.join(projectRoot, "src", "value.js")
					].sort();

				const outputA = await build(rootA, options);
				expect(outputA.code).toContain("value-from-root-a");
				expect(outputA.sources).toEqual(sourcesIn(rootA));

				fs.cpSync(cacheDirectory(rootA), cacheDirectory(rootB), { recursive: true });
				const outputB = await build(rootB, options);
				expect(outputB.code).toContain("value-from-root-b");
				expect(outputB.code).not.toContain("value-from-root-a");
				expect(outputB.sources).toEqual(sourcesIn(rootB));
			} finally {
				fs.rmSync(tempRoot, { recursive: true, force: true, maxRetries: 3 });
			}
		}
	};
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default [
	createCase({ newCache: false, portable: false }),
	createCase({ newCache: true, portable: false }),
	createCase({ newCache: true, portable: true })
];
