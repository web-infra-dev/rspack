const { Processor, parse } = require("postcss");
const pxtorem = require("postcss-pxtorem");
const fs = require("fs");
const path = require("path");

const source = fs.readFileSync(
	path.resolve(__dirname, "../tailwind_component.css"),
).toString();
async function loader(source) {
	// TODO: customize options, until js binding support this functionality
	// console.log(loaderContext.getOptions());
	try {
		console.time("parse");
		let plugins = [
			pxtorem({
				rootValue: 50,
				propList: ["*"],
			}),
		];

		let root = new Processor(plugins);
		let res = await root.process(source);
		// let res = parse(source);
		console.timeEnd("parse");
		return {
			content: res.css,
		};
	} catch (err) {
		throw new Error(err);
	}
}

(async () => {
	for (let i = 0; i < 100; i++) {
		// console.time("label");
		let res = await loader(source);
		// console.timeEnd("label");
	}
})();
