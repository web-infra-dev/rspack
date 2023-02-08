const { Processor } = require("postcss");
const pxtorem = require("postcss-pxtorem");
const cssModules = require("postcss-modules");
const {
	normalizeSourceMap,
	normalizeSourceMapAfterPostcss
} = require("./utils");

const IS_MODULES = /\.module(s)?\.\w+$/i;

module.exports = async function loader(content, sourceMap) {
	const callback = this.async();
	// TODO: customize options, until js binding support this functionality
	// console.log(this.getOptions());
	let options = this.getOptions() ?? {};
	let modulesOptions = options.modules;
	let pxToRem = options.pxToRem;
	let useSourceMap =
		typeof options.sourceMap !== "undefined"
			? options.sourceMap
			: this.sourceMap;
	try {
		let additionalData;
		let plugins = [];
		let enablePxToRem = false;
		let pxToRemConfig = {
			rootValue: 50,
			propList: ["*"]
		};
		let processOptions = {
			from: this.resourcePath,
			to: this.resourcePath
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
				typeof modulesOptions === "object"
					? modulesOptions.auto ?? true
					: undefined;
			let isModules;
			if (typeof auto === "boolean") {
				isModules = auto && IS_MODULES.test(this.resourcePath);
			} else if (auto instanceof RegExp) {
				isModules = auto.test(this.resourcePath);
			} else if (typeof auto === "function") {
				isModules = auto(this.resourcePath);
			} else {
				isModules = true;
			}

			if (isModules) {
				plugins.push(
					cssModules({
						...modulesOptions,
						getJSON(_, json) {
							if (json) {
								additionalData = {
									rspack_postcss_modules: JSON.stringify(json)
								};
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
		if (sourceMap && processOptions.map) {
			processOptions.map.prev = normalizeSourceMap(sourceMap, this.context);
		}

		if (options.postcssOptions?.plugins) {
			plugins.push(...options.postcssOptions.plugins);
		}

		let root = new Processor(plugins);
		let res = await root.process(content, processOptions);
		let map = res.map ? res.map.toJSON() : undefined;

		if (map && useSourceMap) {
			map = normalizeSourceMapAfterPostcss(map, this.context);
		}

		callback(null, res.css, map, additionalData);
	} catch (err) {
		callback(err, "");
	}
};

module.exports.displayName = "postcssLoader";
