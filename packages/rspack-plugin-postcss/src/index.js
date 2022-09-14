const { Processor } = require("postcss");
const pxtorem = require("postcss-pxtorem");
const cssModules = require("postcss-modules");

module.exports = async function loader(loaderContext) {
	// TODO: customize options, until js binding support this functionality
	// console.log(loaderContext.getOptions());
	let options = loaderContext.getOptions() ?? {};
	let enableModules = options.modules;
	let pxToRem = options.pxToRem;
	try {
		let meta = "";
		let plugins = [];
		if (enablePxToRem) {
			plugins.push(
				pxtorem({
					...pxToRem,
				}),
			);
		}
		if (enableModules) {
			plugins.push(
				cssModules({
					getJSON(_, json) {
						if (json) {
							meta = json;
						}
					},
				}),
			);
		}
		let root = new Processor(plugins);
		let res = await root.process(loaderContext.source.getCode(), {
			from: undefined,
		});
		return {
			content: res.css,
			meta: meta ? Buffer.from(JSON.stringify(meta)) : "",
		};
	} catch (err) {
		throw new Error(err);
	}
};
