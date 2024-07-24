const fs = require("fs");
const path = require("path");
const { run } = require("./utils");
const csvToMarkdown = require("csv-to-markdown-table");
const GITHUB_ACTOR = process.env.GITHUB_ACTOR;
const [, , token, commit_sha] = process.argv;
(async () => {
	const rootDir = path.resolve(__dirname, "../../../");
	const currentDataPath = path.resolve(rootDir, "out.json");
	const currentData = JSON.parse(fs.readFileSync(currentDataPath).toString());
	const currentRenderedMdPath = path.resolve(rootDir, "out.md")
	const currentRenderedMd = fs.readFileSync(currentRenderedMdPath, 'utf-8')

	const targetDir = path.resolve(rootDir, ".gh-pages");
	if (!fs.existsSync(targetDir)) {
		await run("git", [
			"clone",
			"--branch",
			"gh-pages",
			"--single-branch",
			"--depth",
			"1",
			token
				? `https://${GITHUB_ACTOR}:${token}@github.com/web-infra-dev/rspack.git`
				: "https://github.com/web-infra-dev/rspack",
			".gh-pages",
		]);
	}
	const cwd = process.cwd();
	let dataPath = path.resolve(targetDir, "result.json");
	let indexPath = path.resolve(targetDir, "index.txt");

	let historyJson = "{}";
	let indexContent = "";
	if (fs.existsSync(dataPath)) {
		historyJson = fs.readFileSync(path.resolve(dataPath)).toString();
	}
	if (fs.existsSync(indexPath)) {
		indexContent = fs.readFileSync(path.resolve(indexPath)).toString();
	}

	process.chdir(targetDir);
	for (let i = 0; i < 21; i++) {
		try {
			await run("git", ["reset", "--hard", "origin/gh-pages"]);
			await run("git", ["pull", "--rebase"]);
			let historyData = JSON.parse(historyJson);
			let indexList = indexContent.split("\n");
			let lastestMainCommit = indexList[indexList.length - 1];

			let latestMainCommitData = historyData[lastestMainCommit];
			console.log(latestMainCommitData);

			let currentCompatibility = currentData["Tests Compatibility"];
			let lastestMainCommitCompatibility =
				latestMainCommitData["Tests Compatibility"];

			if (currentCompatibility !== lastestMainCommitCompatibility) {
				let icon = "❌ ⏬";
				if (currentCompatibility > lastestMainCommitCompatibility) {
					icon = "✅ ⏫";
				}
				let diff =
					+currentCompatibility.slice(0, -1) -
					+lastestMainCommitCompatibility.slice(0, -1);
				let markdown = csvToMarkdown(
					`main,pr,diff
${lastestMainCommitCompatibility},${currentCompatibility},${`${icon} ${diff.toFixed(
						3,
					)}`}
`,
					",",
					true,
				);
				markdown += `
<details>
	<summary>Unpassed tests</summary>

${currentRenderedMd.split('\n').filter(line => !line.includes("🟢")).join('\n')}
</details>
`
				fs.appendFileSync(path.resolve(rootDir, "output.md"), markdown);
			} else {
				fs.rmSync(path.resolve(rootDir, "output.md"));
			}

			break;
		} catch (e) {
			await new Promise((resolve) =>
				setTimeout(resolve, Math.random() * 30000),
			);
			if (i === 20) throw e;
		}
	}
})();

// (async () => {
// 	const targetDir = resolve(rootDir, ".gh-pages");
// 	if (!(await dirExist(targetDir))) {
// 		await run("git", [
// 			"clone",
// 			"--branch",
// 			"gh-pages",
// 			"--single-branch",
// 			"--depth",
// 			"1",
// 			token
// 				? `https://${GITHUB_ACTOR}:${token}@github.com/web-infra-dev/rspack-ecosystem-benchmark.git`
// 				: "https://github.com/web-infra-dev/rspack-ecosystem-benchmark.git",
// 			".gh-pages",
// 		]);
// 	}
// 	process.chdir(targetDir);
// })().catch((err) => {
// 	process.exitCode = 1;
// 	console.error(err.stack);
// });
