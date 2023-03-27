#!/usr/bin/env node
const yargs = require("yargs/yargs");
const { hideBin } = require("yargs/helpers");
const fs = require("fs");
const path = require("path");
const prompts = require("prompts");

const ReNameFiles = {
	_gitignore: ".gitignore"
};

const { formatTargetDir } = require("./utils");
yargs(hideBin(process.argv))
	.command("$0", "init rspack project", async argv => {
		const { help } = argv.argv;
		if (help) return;
		const defaultProjectName = "rspack-project";
		let targetDir = defaultProjectName;
		await prompts([
			{
				type: "text",
				name: "projectDir",
				initial: defaultProjectName,
				message: "Project folder",
				onState: state => {
					targetDir = formatTargetDir(state.value) || defaultProjectName;
				}
			}
		]);
		let root = path.resolve(process.cwd(), targetDir);
		while (fs.existsSync(root)) {
			console.log(
				`${targetDir} is not empty, please choose another project name`
			);
			await prompts([
				{
					type: "text",
					name: "projectDir",
					initial: defaultProjectName,
					message: "Project folder",
					onState: state => {
						targetDir = formatTargetDir(state.value) || defaultProjectName;
					}
				}
			]);
			root = path.resolve(process.cwd(), targetDir);
		}
		fs.mkdirSync(root, { recursive: true });
		// TODO support more template in the future
		const templateDir = "template-react";
		const srcFolder = path.resolve(__dirname, templateDir);
		copyFolder(srcFolder, targetDir);
		const pkgManager = getPkgManager();
		console.log("\nDone. Now run:\n");
		console.log(`cd ${targetDir}\n`);
		console.log(`${pkgManager} install\n`);
		console.log(`${pkgManager} run dev\n`);
	})
	.help()
	.parse();
function copyFolder(src, dst) {
	fs.mkdirSync(dst, { recursive: true });
	for (const file of fs.readdirSync(src)) {
		const srcFile = path.resolve(src, file);
		const dstFile = path.resolve(
			dst,
			ReNameFiles[file] ? ReNameFiles[file] : file
		);
		const stat = fs.statSync(srcFile);
		if (stat.isDirectory()) {
			copyFolder(srcFile, dstFile);
		} else {
			fs.copyFileSync(srcFile, dstFile);
		}
	}
}

function getPkgManager() {
	const ua = process.env.npm_config_user_agent;
	if (!ua) return "npm";
	const [pkgInfo] = ua.split(" ");
	const [name] = pkgInfo.split("/");
	return name || "npm";
}
