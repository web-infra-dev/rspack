module.exports = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.emit.tap({ name: "MyPlugin", stage: 0 }, compilation => {
					console.log("This is an example plugin!");
				});
			}
		}
	]
};
