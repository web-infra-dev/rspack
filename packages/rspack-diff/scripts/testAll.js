const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");
function buildAll() {
	const root = process.cwd();
	for (const dir of fs.readdirSync("./fixtures")) {
		const cwd = path.resolve(root, "fixtures", dir);
		console.log("cwd:", cwd);
		execSync("pnpm build", {
			cwd,
			stdio: "inherit"
		});
	}
}
buildAll();
