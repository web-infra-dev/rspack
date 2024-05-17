import { ApiItemKind,ApiModel } from "@microsoft/api-extractor-model";
import { readdirSync, readFileSync } from "fs";
import { basename, dirname, extname, join,resolve } from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const PLUGIN_REGEX = /^[A-Z][a-zA-Z]+Plugin$/;
const PLUGIN_API_JSON = resolve(__dirname, "../temp/core.api.json");
const PLUGIN_DOCS_DIR = resolve(__dirname, "../../../website/docs/en/plugins");
const INTERNAL_PLUGINS_DOC = join(PLUGIN_DOCS_DIR, "webpack/internal-plugins.mdx");

function getImplementedPlugins() {
	const apiModel = new ApiModel();
	apiModel.loadPackage(PLUGIN_API_JSON);

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

function toCamelCase(s) {
	return s
		.split(/[\s-_]+/)
		.map(word => word[0].toUpperCase() + word.slice(1).toLowerCase())
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
					const name = toCamelCase(basename(item.name, ext));
					if (PLUGIN_REGEX.test(name)) {
						documentedPlugins.add(name);
					}
				}
			}
		}
	}
	visitDir(PLUGIN_DOCS_DIR);

	const internalPluginsDoc = readFileSync(INTERNAL_PLUGINS_DOC, 'utf-8');
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
