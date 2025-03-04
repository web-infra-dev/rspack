const path = require("node:path");
const fs = require("fs-extra");
const { describeByWalk } = require("./dist");

const TEST_DIR = path.resolve(__dirname, "./tests");
const BATCH_CONFIG = {
	"Watch.template.js": {
		tasks: 10
	},
	"NewIncremental-watch.template.js": {
		tasks: 10,
		walkConfig: {
			source: path.resolve(TEST_DIR, "./watchCases"),
			dist: path.resolve(TEST_DIR, "./js/new-incremental/watch")
		}
	},
	"NewIncremental-watch-webpack.template.js": {
		tasks: 10,
		walkConfig: {
			source: path.resolve(TEST_DIR, "../../../tests/webpack-test/watchCases"),
			dist: path.resolve(TEST_DIR, "./js/new-incremental/webpack-test/watch")
		}
	}
};

const templates = fs
	.readdirSync(TEST_DIR)
	.filter(file => file.endsWith(".template.js"));
for (const template of templates) {
	const batchConfig = BATCH_CONFIG[template];
	if (!batchConfig) {
		continue;
	}
	const templateContent = fs.readFileSync(
		path.resolve(TEST_DIR, template),
		"utf-8"
	);
	const testFileName = template.replace(".template.js", ".test.js");
	const testDistDir = path.resolve(
		TEST_DIR,
		testFileName.replace(".test.js", "-splitted")
	);
	if (fs.existsSync(testDistDir)) {
		fs.removeSync(testDistDir);
	}
	const cases = [];
	describeByWalk(
		path.join(TEST_DIR, testFileName),
		(name, src, dist) => {
			cases.push({
				name,
				src,
				dist
			});
		},
		{
			describe: (name, fn) => {
				fn();
			},
			...(batchConfig.walkConfig || {})
		}
	);

	const [header, repeat, footer] = templateContent.split(
		/\/\* (?:start|end) each case \*\//g
	);
	const result = `
${header}
${cases
	.map(c =>
		repeat
			.replace(/\$name\$/g, JSON.stringify(c.name))
			.replace(/\$src\$/g, JSON.stringify(c.src))
			.replace(/\$dist\$/g, JSON.stringify(c.dist))
	)
	.join("\n")}
${footer}
  `;

	const { tasks } = batchConfig;
	const batchCases = [];
	for (let i = 0; i < cases.length; i += tasks) {
		batchCases.push(cases.slice(i, i + tasks));
	}

	for (const [index, batchCase] of Object.entries(batchCases)) {
		const result = `
${header}
${batchCase
	.map(c =>
		repeat
			.replace(/\$name\$/g, JSON.stringify(c.name))
			.replace(/\$src\$/g, JSON.stringify(c.src))
			.replace(/\$dist\$/g, JSON.stringify(c.dist))
	)
	.join("\n")}
${footer}
  `;
		fs.ensureDirSync(testDistDir);
		fs.writeFileSync(
			path.resolve(testDistDir, `${index}-${testFileName}`),
			result
		);
	}
}
