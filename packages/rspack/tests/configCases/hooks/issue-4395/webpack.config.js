const path = require("path");
const fs = require("fs");

const config = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.make.tap("child", base => {
					const child = base.createChildCompiler("child", {}, []);
					child.runAsChild(() => {});
				});
			}
		},
		{
			apply(compiler) {
				let count = 0;
				const add = () => {
					count += 1;
				};
				const sub = () => {
					count -= 1;
				};
				compiler.hooks.thisCompilation.tap("mock-plugin", compilation => {
					compilation.hooks.processAssets.tap("mock-plugin", () => {
						sub();
						if (count === 0) {
							const temp = path.resolve(__dirname, "dist/temp");
							fs.writeFileSync(temp, "");
						}
					});
				});
				compiler.hooks.run.tap("mock-plugin", add);
			}
		}
	]
};

module.exports = config;
