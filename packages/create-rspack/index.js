#!/usr/bin/env node
const yargs = require("yargs/yargs");
const { hideBin } = require("yargs/helpers");
const fs = require("fs");
const path = require("path");
const prompts = require("prompts");
yargs(hideBin(process.argv))
	.command("$0", "init rspack project", argv => {
		const { help } = argv.argv;
		if (help) return;
		const defaultProjectName = "rspack-project";
		prompts([
			{
				type: "text",
				name: "projectDir",
				initial: defaultProjectName,
				message: "Project folder"
			}
		]).then(result => {
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
			console.log("\nDone. Now run:\n");
			console.log(`cd ${projectDir}\n`);
			console.log(`npm install\n`);
			console.log(`npm run dev\n`);
		});
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
