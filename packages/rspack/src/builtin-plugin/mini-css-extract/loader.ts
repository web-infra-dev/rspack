import schema from "./loader-options.json";
import { RspackCssExtractPlugin } from "./index";
import path from "path";
import { stringifyLocal, stringifyRequest } from "./utils";

import type { LoaderContext, LoaderDefinition } from "../..";

export const MODULE_TYPE = "css/mini-extract";
export const AUTO_PUBLIC_PATH = "__mini_css_extract_plugin_public_path_auto__";
export const ABSOLUTE_PUBLIC_PATH = "webpack:///mini-css-extract-plugin/";
export const BASE_URI = "webpack://";
export const SINGLE_DOT_PATH_SEGMENT =
	"__mini_css_extract_plugin_single_dot_path_segment__";

const SERIALIZE_SEP = "__RSPACK_CSS_EXTRACT_SEP__";

interface DependencyDescription {
	identifier: string;
	content: string;
	context: string;
	media?: string;
	supports?: string;
	layer?: string;
	sourceMap?: string;
	identifierIndex: number;
	filepath: string;
}

export interface LoaderOptions {
	publicPath?: string | ((resourcePath: string, context: string) => string);
	emit?: boolean;
	esModule?: boolean;

	// TODO: support layer
	layer?: boolean;
}

function hotLoader(
	content: string,
	context: {
		loaderContext: LoaderContext;
		options: LoaderOptions;
		locals: Record<string, string>;
	}
) {
	const accept = context.locals
		? ""
		: "module.hot.accept(undefined, cssReload);";
	return `${content}
    if(module.hot) {
      // ${Date.now()}
      var cssReload = require(${stringifyRequest(
				context.loaderContext,
				path.join(__dirname, "./hmr/hotModuleReplacement.js")
			)})(module.id, ${JSON.stringify({
		...context.options,
		locals: !!context.locals
	})});
      module.hot.dispose(cssReload);
      ${accept}
    }
  `;
}

// mini-css-extract-plugin
const loader: LoaderDefinition = function loader(content) {
	if (
		this._compiler &&
		this._compiler.options &&
		this._compiler.options.experiments &&
		this._compiler.options.experiments.css
	) {
		return content;
	}
};

export const pitch: LoaderDefinition["pitch"] = function (request, _, data) {
	if (
		this._compiler &&
		this._compiler.options &&
		this._compiler.options.experiments &&
		this._compiler.options.experiments.css
	) {
		let e = new Error(
			"You can't use `experiments.css` and `mini-css-extract-plugin` together, please set `experiments.css` to `false`"
		);
		e.stack = undefined;
		this.emitWarning(e);

		return;
	}

	const options = this.getOptions(schema) as LoaderOptions;
	const emit = typeof options.emit !== "undefined" ? options.emit : true;
	const callback = this.async();
	const filepath = this.resourcePath;

	let { publicPath } =
		/** @type {Compilation} */
		this._compilation.outputOptions;

	if (typeof options.publicPath === "string") {
		// eslint-disable-next-line prefer-destructuring
		publicPath = options.publicPath;
	} else if (typeof options.publicPath === "function") {
		publicPath = options.publicPath(this.resourcePath, this.rootContext);
	}

	if (publicPath === "auto") {
		publicPath = AUTO_PUBLIC_PATH;
	}

	let publicPathForExtract: string | undefined;

	if (typeof publicPath === "string") {
		const isAbsolutePublicPath = /^[a-zA-Z][a-zA-Z\d+\-.]*?:/.test(publicPath);

		publicPathForExtract = isAbsolutePublicPath
			? publicPath
			: `${ABSOLUTE_PUBLIC_PATH}${publicPath.replace(
					/\./g,
					SINGLE_DOT_PATH_SEGMENT
			  )}`;
	} else {
		publicPathForExtract = publicPath;
	}

	const handleExports = (
		originalExports:
			| { default: Record<string, any>; __esModule: true }
			| Record<string, any>
	) => {
		/** @type {Locals | undefined} */
		let locals: Record<string, string>;
		let namedExport;

		const esModule =
			typeof options.esModule !== "undefined" ? options.esModule : true;
		let dependencies: DependencyDescription[] = [];

		try {
			// eslint-disable-next-line no-underscore-dangle
			const exports = originalExports.__esModule
				? originalExports.default
				: originalExports;

			namedExport =
				// eslint-disable-next-line no-underscore-dangle
				originalExports.__esModule &&
				(!originalExports.default || !("locals" in originalExports.default));

			if (namedExport) {
				Object.keys(originalExports).forEach(key => {
					if (key !== "default") {
						if (!locals) {
							locals = {};
						}

						/** @type {Locals} */ locals[key] = (
							originalExports as Record<string, string>
						)[key];
					}
				});
			} else {
				locals = exports && exports.locals;
			}

			if (Array.isArray(exports) && emit) {
				const identifierCountMap = new Map();

				dependencies = exports
					.map(([id, content, media, sourceMap, supports, layer]) => {
						let identifier = id;
						let context = this.rootContext;

						const count = identifierCountMap.get(identifier) || 0;

						identifierCountMap.set(identifier, count + 1);

						return {
							identifier,
							context,
							content,
							media,
							supports,
							layer,
							identifierIndex: count,
							sourceMap: sourceMap
								? JSON.stringify(sourceMap)
								: // eslint-disable-next-line no-undefined
								  undefined,
							filepath
						};
					})
					.filter(item => item !== null) as DependencyDescription[];
			}
		} catch (e) {
			callback(e as Error);

			return;
		}

		const result = locals!
			? namedExport
				? Object.keys(locals)
						.map(
							key =>
								`\nexport var ${key} = ${stringifyLocal(
									/** @type {Locals} */ locals[key]
								)};`
						)
						.join("")
				: `\n${
						esModule ? "export default" : "module.exports ="
				  } ${JSON.stringify(locals)};`
			: esModule
			? `\nexport {};`
			: "";

		let resultSource = `// extracted by ${RspackCssExtractPlugin.pluginName}`;

		// only attempt hotreloading if the css is actually used for something other than hash values
		resultSource +=
			this.hot && emit
				? hotLoader(result, { loaderContext: this, options, locals: locals! })
				: result;

		const additionalData: Record<string, any> = { ...data };
		if (dependencies.length > 0) {
			additionalData[RspackCssExtractPlugin.pluginName] = dependencies
				.map(dep => {
					return [
						dep.identifier,
						dep.content,
						dep.context,
						dep.media,
						dep.supports,
						dep.sourceMap,
						dep.identifierIndex,
						dep.filepath
					].join(SERIALIZE_SEP);
				})
				.join(SERIALIZE_SEP);
		}

		callback(null, resultSource, undefined, additionalData);
	};

	this.importModule(
		`${this.resourcePath}.webpack[javascript/auto]!=!!!${request}`,
		{
			publicPath: /** @type {string} */ publicPathForExtract,
			baseUri: `${BASE_URI}/`
		},
		(error, exports) => {
			if (error) {
				callback(error);

				return;
			}

			handleExports(exports);
		}
	);
};

export default loader;
