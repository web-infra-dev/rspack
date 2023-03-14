#!/usr/bin/env node
const yargs = require("yargs/yargs");
const { hideBin } = require("yargs/helpers");
const fs = require("fs");
const path = require("path");
const prompts = require("prompts");
yargs(hideBin(process.argv))
	.command("$0", "init rspack project", async argv => {
		const defaultProjectName = "rspack-project";
		let result = await prompts([
			{
				type: "text",
				name: "projectDir",
				initial: defaultProjectName,
				message: "Project folder"
			}
		]);
		const { projectDir } = result;
		const root = path.resolve(process.cwd(), projectDir);
		if (fs.existsSync(root)) {
			throw new Error("project directory already exists");
		}
		fs.mkdirSync(root);
		// TODO support more template in the future
		const templateDir = "template-react";
		const srcFolder = path.resolve(__dirname, templateDir);
		copyFolder(srcFolder, projectDir);
		const pkgInfo = pkgFromUserAgent(process.env.npm_config_user_agent);
		const pkgManager = pkgInfo ? pkgInfo.name : "npm";
		console.log("\nDone. Now run:\n");
		console.log(`cd ${projectDir}\n`);
		switch (pkgManager) {
			case "yarn":
				console.log("yarn");
				console.log("yarn dev");
				break;
			default:
				console.log(`${pkgManager} install\n`);
				console.log(`${pkgManager} run dev\n`);
				break;
		}
	})
	.help()
	.parse();
function copyFolder(src, dst) {
	fs.mkdirSync(dst, { recursive: true });
	for (const file of fs.readdirSync(src)) {
		const srcFile = path.resolve(src, file);
		const dstFile = path.resolve(dst, file);
		const stat = fs.statSync(srcFile);
		if (stat.isDirectory()) {
			copyFolder(srcFile, dstFile);
		} else {
			fs.copyFileSync(srcFile, dstFile);
		}
	}
}

function pkgFromUserAgent(userAgent) {
	if (!userAgent) return undefined;
	const pkgSpec = userAgent.split(" ")[0];
	const pkgSpecArr = pkgSpec.split("/");
	return {
		name: pkgSpecArr[0],
		version: pkgSpecArr[1]
	};
}
