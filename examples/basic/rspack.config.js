		const { util } = compiler.webpack;
		console.log(typeof util.createHash);
		let content = "something else";
		let hash = util.createHash("xxhash64").update(content).digest();
		console.log(hash);
	}
}
/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	},
	plugins: [new Plugin()]
};
