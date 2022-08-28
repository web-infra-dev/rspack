const { Processor } = require("postcss");
const pxtorem = require("postcss-pxtorem");
const cssModules = require("postcss-modules");

module.exports = async function loader(loaderContext) {
	// TODO: customize options, until js binding support this functionality
	console.log(loaderContext.getOptions());
	let options = loaderContext.getOptions() ?? {};
	let enableModules = options.modules;
	try {
		let meta = "";
		let plugins = [pxtorem];
		if (enableModules) {
			plugins.push(
				cssModules({
					getJSON(name, json) {
						if (json) {
							meta = json;
						}
					},
				}),
			);
		}
		let root = new Processor(plugins);
		let res = await root.process(loaderContext.source.getCode());
		return {
			content: res.css,
			metaData: meta ? Buffer.from(JSON.stringify(meta)) : "",
		};
	} catch (err) {
		throw new Error(err);
	}
};
