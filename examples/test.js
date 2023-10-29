const fs = require("fs");

const migrate = dir => {
	if (fs.statSync(dir).isFile() || dir.includes("node_modules")) return;
	const source = JSON.parse(fs.readFileSync(`${dir}/package.json`, "utf-8"));
	if (!source.devDependencies?.["@rspack/core"]) {
		(source.devDependencies ||= {})["@rspack/core"] = "workspace:*";
	}
	fs.writeFileSync(`${dir}/package.json`, JSON.stringify(source, null, 2));
	let config = fs.readFileSync(`${dir}/rspack.config.js`, "utf-8");
	if (!config.includes(`const rspack = require("@rspack/core")`)) {
		config = `const rspack = require("@rspack/core");\n` + config;
		fs.writeFileSync(`${dir}/rspack.config.js`, config);
	}
};

fs.readdirSync(__dirname).forEach(migrate);
