const { Processor } = require("postcss");
const pxtorem = require("postcss-pxtorem");
const cssModules = require("postcss-modules");
const {
	normalizeSourceMap,
	normalizeSourceMapAfterPostcss
} = require("./utils");

const IS_MODULES = /\.module(s)?\.\w+$/i;

module.exports = async function loader(loaderContext) {
	// TODO: customize options, until js binding support this functionality
	// console.log(loaderContext.getOptions());
	let options = loaderContext.getOptions() ?? {};
	let modulesOptions = options.modules;
	let pxToRem = options.pxToRem;
	let useSourceMap =
		typeof options.sourceMap !== "undefined"
			? options.sourceMap
			: loaderContext.useSourceMap;
	try {
		let meta = "";
		let plugins = [];
		let enablePxToRem = false;
		let pxToRemConfig = {
			rootValue: 50,
			propList: ["*"]
		};
		let processOptions = {
			from: loaderContext.resourcePath,
			to: loaderContext.resourcePath
		};

		if (pxToRem) {
			enablePxToRem = true;
			// Custom config
			if (typeof pxToRem === "object") {
				pxToRemConfig = pxToRem;
			}
		}

		if (enablePxToRem) {
			plugins.push(pxtorem(pxToRemConfig));
		}

		if (modulesOptions) {
			let auto =
				typeof modulesOptions === "boolean" ? true : modulesOptions.auto;
			let isModules;
			if (typeof auto === "boolean") {
				isModules = auto && IS_MODULES.test(loaderContext.resourcePath);
			} else if (auto instanceof RegExp) {
				isModules = auto.test(loaderContext.resourcePath);
			} else if (typeof auto === "function") {
				isModules = auto(loaderContext.resourcePath);
			}
			delete modulesOptions.auto;

			if (isModules) {
				plugins.push(
					cssModules({
						...modulesOptions,
						getJSON(_, json) {
							if (json) {
								meta = json;
							}
						}
					})
				);
			}
		}

		if (useSourceMap) {
			processOptions.map = {
				inline: false,
				annotation: false
			};
		}
		if (loaderContext.sourceMap && processOptions.map) {
			processOptions.map.prev = normalizeSourceMap(
				loaderContext.sourceMap,
				loaderContext.context
			);
		}

		let root = new Processor(plugins);
		let res = await root.process(
			loaderContext.source.getCode(),
			processOptions
		);
		let map = res.map ? res.map.toJSON() : undefined;

		if (map && useSourceMap) {
			map = normalizeSourceMapAfterPostcss(map, loaderContext.context);
		}

		return {
			content: res.css,
			meta: meta ? Buffer.from(JSON.stringify(meta)) : "",
			sourceMap: map
		};
	} catch (err) {
		throw new Error(err);
	}
};
