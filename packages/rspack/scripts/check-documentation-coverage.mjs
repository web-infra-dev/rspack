import { readFileSync, readdirSync } from "fs";
import { basename, dirname, extname, join, resolve } from "path";
import { fileURLToPath } from "url";
import { ApiItemKind, ApiModel } from "@microsoft/api-extractor-model";
import { ZodObject, ZodOptional, ZodUnion } from "../compiled/zod/index.js";
import { rspackOptions } from "../dist/config/zod.js";

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
	const headings = [];
	let match;
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

	const undocumentedPlugins = Array.from(implementedPlugins).filter(
		plugin => !documentedPlugins.has(plugin)
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

	function getImplementedConfigs() {
		const implementedConfigs = [];
		function visit(zod, path = "") {
			if (zod instanceof ZodObject) {
				for (const [key, schema] of Object.entries(zod.shape)) {
					const next = (() => {
						if (key.includes("/")) {
							return path + `["${key}"]`;
						} else {
							if (path) {
								return path + "." + key;
							}
							return key;
						}
					})();
					implementedConfigs.push(next);
					visit(schema, next);
				}
			} else if (zod instanceof ZodOptional) {
				visit(zod.unwrap(), path);
			} else if (zod instanceof ZodUnion) {
				for (let schema of zod.options) {
					visit(schema, path);
				}
			}
		}
		visit(rspackOptions);
		return implementedConfigs;
	}

	function parseConfigDocuments() {
		function parseMarkdownContent(content) {
			const sections = [];
			let section;
			const lines = content.split("\n");
			for (let i = 0; i < lines.length; i++) {
				const line = lines[i];
				if (line.startsWith("#")) {
					let level;
					for (let j = 0; j < line.length; j++) {
						if (level === undefined) {
							if (line[j] != "#") {
								level = j;
							}
						} else {
							break;
						}
					}
					const title = line.substring(level).trim();
					section = {
						title: toCamelCase(title.split(' ')[0]),
						level,
						text: ""
					};
					sections.push(section);
				} else if (section) {
					section.text += line;
				}
			}
			return sections;
		}

		const sections = [];
		function visitDir(dir) {
			const items = readdirSync(dir, { withFileTypes: true });
			for (const item of items) {
				const resPath = resolve(dir, item.name);
				if (item.isDirectory()) {
					visitDir(resPath);
				} else {
					const ext = extname(item.name);
					if (ext === ".mdx") {
						try {
							const content = readFileSync(join(item.path, item.name), "utf-8");
							const markdownBlocks = parseMarkdownContent(content);
							sections.push(...markdownBlocks);
						} catch (error) {
							console.log('error: ', error);
							console.log('item: ', item);
						}
					}
				}
			}
		}
		visitDir(CONFIG_DOCS_DIR);
		return sections;
	}

	const implementedConfigs = getImplementedConfigs().filter(config => {
		return ![
			"resolveLoader",
			"module",
			"experiments.rspackFuture",

			"output.library.amd",
			"output.library.commonjs",
			"output.library.root",
			"output.environment.asyncFunction",
			"output.environment.bigIntLiteral",
			"output.environment.const",
			"output.environment.destructuring",
			"output.environment.document",
			"output.environment.dynamicImport",
			"output.environment.dynamicImportInWorker",
			"output.environment.forOf",
			"output.environment.globalThis",
			"output.environment.module",
			"output.environment.nodePrefixForCoreModules",
			"output.environment.optionalChaining",
			"output.environment.templateLiteral",
			"output.workerChunkLoading",
			"output.workerWasmLoading",
			"output.workerPublicPath",
			"output.strictModuleExceptionHandling",
			"output.sourceMapFilename",

			"node",
			"stats",

			"optimization.splitChunks",
			"optimization.removeAvailableModules",
			"optimization.concatenateModules",

			"loader",
			"snapshot",
			"profile"
		].some(c => config.startsWith(c));
	});
	const markdownSections = parseConfigDocuments();

	const undocumentedConfigs = [];
	const map = new Map();
	for (const config of implementedConfigs) {
		let documented = false;
		for (const section of markdownSections) {
			if (section.title === config) {
				documented = true;
				map.set(config, section);
			}
		}
		if (!documented) {
			const parts = config.split(".");
			const subs = [];
			let part;
			while ((part = parts.pop())) {
				subs.push(part);
				const section = map.get(parts.join("."));
				if (section) {
					if (subs.every(sub => section.text.includes(sub))) {
						documented = true;
						break;
					}
				}
			}
		}
		if (!documented) {
			undocumentedConfigs.push(config);
		}
	}

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
