import { readFileSync, readdirSync } from "node:fs";
import { basename, dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { ApiItemKind, ApiModel } from "@microsoft/api-extractor-model";
// import { ZodObject, ZodOptional, ZodUnion } from "zod";
// import { rspackOptions } from "../src/config/zod.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const CORE_API_JSON = resolve(__dirname, "../temp/core.api.json");

function toPascalCase(s) {
	return s
		.split(/[\s-_]+/)
		.map(word => word[0].toUpperCase() + word.slice(1).toLowerCase())
		.join("");
}

function toCamelCase(s) {
	return s
		.split(/[\s-_]+/)
		.map(
			(word, index) =>
				(index === 0 ? word[0].toLowerCase() : word[0].toUpperCase()) +
				word.slice(1)
		)
		.join("");
}

function extractMarkdownHeadings(markdown) {
	const headingRegex = /^(#{1,6})\s+(.+)$/gm;
	const headings: string[] = [];
	let match: RegExpExecArray | null;
	while ((match = headingRegex.exec(markdown)) !== null) {
		headings.push(match[2].trim());
	}
	return headings;
}

function checkPluginsDocumentationCoverage() {
	const PLUGIN_REGEX = /^[A-Z][a-zA-Z]+Plugin$/;
	const PLUGIN_DOCS_DIR = resolve(
		__dirname,
		"../../../website/docs/en/plugins"
	);
	const INTERNAL_PLUGINS_DOC = join(
		PLUGIN_DOCS_DIR,
		"webpack/internal-plugins.mdx"
	);

	function getImplementedPlugins() {
		const apiModel = new ApiModel();
		apiModel.loadPackage(CORE_API_JSON);

		const implementedPlugins = new Set();

		function visitApiItem(apiItem) {
			if (
				[ApiItemKind.Class, ApiItemKind.Variable].includes(apiItem.kind) &&
				PLUGIN_REGEX.test(apiItem.displayName) &&
				!apiItem.isAbstract
			) {
				implementedPlugins.add(apiItem.displayName);
			}
			for (const member of apiItem.members) {
				visitApiItem(member);
			}
		}
		visitApiItem(apiModel);

		return implementedPlugins;
	}

	function getDocumentedPlugins() {
		const documentedPlugins = new Set();

		function visitDir(dir) {
			const items = readdirSync(dir, { withFileTypes: true });
			for (const item of items) {
				const resPath = resolve(dir, item.name);
				if (item.isDirectory()) {
					visitDir(resPath);
				} else {
					const ext = extname(item.name);
					if (ext === ".mdx") {
						const name = toPascalCase(basename(item.name, ext));
						if (PLUGIN_REGEX.test(name)) {
							documentedPlugins.add(name);
						}
					}
				}
			}
		}
		visitDir(PLUGIN_DOCS_DIR);

		const internalPluginsDoc = readFileSync(INTERNAL_PLUGINS_DOC, "utf-8");
		const headings = extractMarkdownHeadings(internalPluginsDoc);
		for (const heading of headings) {
			if (PLUGIN_REGEX.test(heading)) {
				documentedPlugins.add(heading);
			}
		}

		return documentedPlugins;
	}

	const implementedPlugins = getImplementedPlugins();
	const documentedPlugins = getDocumentedPlugins();

	const excludedPlugins = [
		"OriginEntryPlugin",
		"RuntimePlugin", // This plugin only provides hooks, should not be used separately
		"RsdoctorPlugin" // This plugin is not stable yet
	];

	const undocumentedPlugins = Array.from(implementedPlugins).filter(
		plugin =>
			!documentedPlugins.has(plugin) &&
			!excludedPlugins.includes(plugin as string)
	);
	const unimplementedPlugins = Array.from(documentedPlugins).filter(
		plugin => !implementedPlugins.has(plugin)
	);

	if (undocumentedPlugins.length) {
		console.error(
			"The following plugins are implemented but not documented:",
			undocumentedPlugins.join(", ")
		);
	}

	if (unimplementedPlugins.length) {
		if (undocumentedPlugins.length) {
			console.log("\n");
		}
		console.error(
			"The following plugins are documented but not implemented or not properly exported:",
			unimplementedPlugins.join(", ")
		);
	}

	if (undocumentedPlugins.length || unimplementedPlugins.length) {
		process.exit(1);
	}
}

type Section = {
	title: string;
	level: number;
	text: string;
};

/**
 * The process of checking the documentation coverage of Rspack configuration
 *
 * 1. Retrieve and traverse all implemented Rspack configurations through zod declaration.
 * 2. Traverse the configurations and determine whether they match the any level titles of the Markdown files under the config directory of the document site:
 *     1. If so, pass.
 *     2. If not, judge whether the introduction of the configuration exists in the body of the parent configuration:
 *       1. If so, pass.
 *       2. If not, fail.
 */
function checkConfigsDocumentationCoverage() {
	const CONFIG_DOCS_DIR = resolve(__dirname, "../../../website/docs/en/config");

	// function getImplementedConfigs() {
	// 	const implementedConfigs: string[] = [];
	// 	function visit(zod, path = "") {
	// 		if (zod instanceof ZodObject) {
	// 			for (const [key, schema] of Object.entries(zod.shape)) {
	// 				const next = (() => {
	// 					if (key.includes("/")) {
	// 						return `${path}["${key}"]`;
	// 					}
	// 					if (path) {
	// 						return `${path}.${key}`;
	// 					}
	// 					return key;
	// 				})();
	// 				implementedConfigs.push(next);
	// 				visit(schema, next);
	// 			}
	// 		} else if (zod instanceof ZodOptional) {
	// 			visit(zod.unwrap(), path);
	// 		} else if (zod instanceof ZodUnion) {
	// 			for (const schema of zod.options) {
	// 				visit(schema, path);
	// 			}
	// 		}
	// 	}
	// 	visit(rspackOptions);
	// 	return implementedConfigs;
	// }

	function parseConfigDocuments() {
		function parseMarkdownContent(content) {
			const sections: Section[] = [];
			let section: Section | null = null;

			const lines = content.split("\n");

			for (let i = 0; i < lines.length; i++) {
				const line = lines[i];
				if (line.startsWith("#")) {
					let level: number | undefined;
					for (let j = 0; j < line.length; j++) {
						if (level === undefined) {
							if (line[j] !== "#") {
								level = j;
							}
						} else {
							break;
						}
					}
					const title = line
						.substring(level)
						.trim()
						.split(" ")[0]
						.replace(/\\/g, "");
					section = {
						title: title.includes(".") ? title : toCamelCase(title),
						level: level!,
						text: ""
					};
					sections.push(section!);
				} else if (section) {
					section.text += line;
				}
			}
			return sections;
		}

		const sections: Section[] = [];
		function visitDir(dir) {
			const items = readdirSync(dir, { withFileTypes: true });
			for (const item of items) {
				const resPath = resolve(dir, item.name);
				if (item.isDirectory()) {
					visitDir(resPath);
				} else {
					const ext = extname(item.name);
					if (ext === ".mdx") {
						const content = readFileSync(join(dir, item.name), "utf-8");
						const markdownBlocks = parseMarkdownContent(content);
						sections.push(...markdownBlocks);
					}
				}
			}
		}
		visitDir(CONFIG_DOCS_DIR);
		return sections;
	}

	// const implementedConfigs = getImplementedConfigs().filter(config => {
	// 	return ![
	// 		"experiments.lazyCompilation.backend",
	// 		"resolveLoader",
	// 		"module.parser",
	// 		"module.generator",
	// 		"experiments.rspackFuture",
	// 		"experiments.incremental",
	// 		"output.library.amd",
	// 		"output.library.commonjs",
	// 		"output.library.root",
	// 		"output.workerChunkLoading",
	// 		"output.workerWasmLoading",
	// 		"output.workerPublicPath",
	// 		"output.strictModuleExceptionHandling",
	// 		"output.auxiliaryComment.amd",
	// 		"output.auxiliaryComment.commonjs",
	// 		"output.auxiliaryComment.commonjs2",
	// 		"output.auxiliaryComment.root",
	// 		"stats",
	// 		"optimization.splitChunks",
	// 		"optimization.removeAvailableModules",
	// 		"optimization.concatenateModules",
	// 		"loader",
	// 		"snapshot",
	// 		"profile"
	// 	].some(c => config.startsWith(c));
	// });
	const markdownSections = parseConfigDocuments();
	const undocumentedConfigs: string[] = [];
	const map = new Map();

	// for (const config of implementedConfigs) {
	// 	let documented = false;
	// 	for (const section of markdownSections) {
	// 		if (section.title === config) {
	// 			documented = true;
	// 			map.set(config, section);
	// 		}
	// 	}
	// 	if (!documented) {
	// 		const parts = config.split(".");
	// 		const subs: string[] = [];
	// 		let part: string | undefined;
	// 		while ((part = parts.pop())) {
	// 			subs.push(part);
	// 			const section = map.get(parts.join("."));
	// 			if (section) {
	// 				if (subs.every(sub => section.text.includes(sub))) {
	// 					documented = true;
	// 					break;
	// 				}
	// 			}
	// 		}
	// 	}
	// 	if (!documented) {
	// 		undocumentedConfigs.push(config);
	// 	}
	// }

	if (undocumentedConfigs.length) {
		console.error(
			"The following configs are implemented but not documented:",
			undocumentedConfigs.join(", ")
		);
	}

	if (undocumentedConfigs.length) {
		process.exit(1);
	}
}

checkPluginsDocumentationCoverage();
checkConfigsDocumentationCoverage();
