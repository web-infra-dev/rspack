#!/usr/bin/env node
const yargs = require("yargs/yargs");
const { hideBin } = require("yargs/helpers");
const fs = require("fs");
const path = require("path");
const prompts = require("prompts");

const templateMap = {
	react: "template-react",
	vue: "template-vue"
};
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
		const { templateName } = await prompts([
			{
				type: "select",
				name: "templateName",
				choices: Object.keys(templateMap).map(key => ({
					title: key,
					value: key
				})),
				message: "Project framework"
			}
		]);
		// TODO support more template in the future
		const templateDir = templateMap[templateName];

		const srcFolder = path.resolve(__dirname, templateDir);
		copyFolder(srcFolder, projectDir);
		console.log("\nDone. Now run:\n");
		console.log(`cd ${projectDir}\n`);
		console.log(`npm install\n`);
		console.log(`npm run dev\n`);
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
