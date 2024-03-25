#!/usr/bin/env node
const yargs = require("yargs/yargs");
const { hideBin } = require("yargs/helpers");
const fs = require("fs");
const path = require("path");
const prompts = require("prompts");
const { formatTargetDir } = require("./utils");
const { version } = require("./package.json");

yargs(hideBin(process.argv))
	.command("$0", "init rspack project", async argv => {
		const { help } = argv.argv;
		if (help) return;

		const onCancel = () => {
			console.log("Operation cancelled.");
			process.exit(0);
		};

		const defaultProjectName = "rspack-project";
		let template = "react";
		let targetDir = defaultProjectName;
		const promptProjectDir = async () =>
			await prompts(
				[
					{
						type: "text",
						name: "projectDir",
						initial: defaultProjectName,
						message: "Project folder",
						onState: state => {
							targetDir = formatTargetDir(state.value) || defaultProjectName;
						}
					}
				],
				{ onCancel }
			);

		await promptProjectDir();
		let root = path.resolve(process.cwd(), targetDir);
		while (fs.existsSync(root)) {
			console.log(
				`${targetDir} is not empty, please choose another project name`
			);
			await promptProjectDir();
			root = path.resolve(process.cwd(), targetDir);
		}

		// choose template
		await prompts(
			[
				{
					type: "select",
					name: "template",
					message: "Project template",
					choices: [
						{ title: "react", value: "react" },
						{ title: "react-ts", value: "react-ts" },
						{ title: "vue", value: "vue" }
					],
					onState: state => {
						template = state.value;
					}
				}
			],
			{ onCancel }
		);

		fs.mkdirSync(root, { recursive: true });
		const srcFolder = path.resolve(__dirname, `template-${template}`);
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
	const renameFiles = {
		_gitignore: ".gitignore"
	};

	fs.mkdirSync(dst, { recursive: true });
	for (const file of fs.readdirSync(src)) {
		if (file === "node_modules") {
			continue;
		}
		const srcFile = path.resolve(src, file);
		const dstFile = renameFiles[file]
			? path.resolve(dst, renameFiles[file])
			: path.resolve(dst, file);
		const stat = fs.statSync(srcFile);
		if (stat.isDirectory()) {
			copyFolder(srcFile, dstFile);
		} else {
			// use create-rspack version as @rspack/xxx version in template
			if (file === "package.json") {
				const pkg = require(srcFile);
				if (pkg.dependencies) {
					for (const key of Object.keys(pkg.dependencies)) {
						if (key.startsWith("@rspack/")) {
							pkg.dependencies[key] = version;
						}
					}
				}
				if (pkg.devDependencies) {
					for (const key of Object.keys(pkg.devDependencies)) {
						if (key.startsWith("@rspack/")) {
							pkg.devDependencies[key] = version;
						}
					}
				}
				fs.writeFileSync(dstFile, JSON.stringify(pkg, null, 2), "utf-8");
			} else {
				fs.copyFileSync(srcFile, dstFile);
			}
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
