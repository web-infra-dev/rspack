// Diagnostic probe for https://github.com/web-infra-dev/rspack/issues/15352
// Runs `compiler.watch()` with a persistent filesystem cache and reports
//   1. whether a file change triggers a rebuild
//   2. the exact spelling of every path in `compilation.fileDependencies`
// Exit code is always 0: the verdict lives in the log so CI keeps running the matrix.
const fs = require('node:fs');
const path = require('node:path');
const { rspack } = require('@rspack/core');

const PHASE = process.env.PHASE || 'cold';
const TOUCH = process.env.TOUCH || 'dep';
const PORTABLE = process.env.PORTABLE === '1';
const CLEAN = process.env.CLEAN === '1';

const root = __dirname;
const cacheDir = path.resolve(root, 'node_modules/.cache/rspack');
const version = require('@rspack/core/package.json').version;
const tag = `version=${version} phase=${PHASE} touch=${TOUCH} portable=${PORTABLE ? 1 : 0}`;

if (CLEAN) {
	fs.rmSync(cacheDir, { recursive: true, force: true });
	fs.rmSync(path.resolve(root, 'dist'), { recursive: true, force: true });
	console.log(`[clean] removed cache + dist`);
}

const compiler = rspack({
	mode: 'development',
	context: root,
	entry: './src/index.js',
	output: { path: path.resolve(root, 'dist') },
	cache: {
		type: 'persistent',
		portable: PORTABLE,
		storage: { type: 'filesystem', directory: cacheDir }
	}
});

const isWin = process.platform === 'win32';
const startedAt = Date.now();
let builds = 0;

const watching = compiler.watch({}, (err, stats) => {
	builds++;
	if (err) {
		console.log(`[build ${builds}] ERROR ${err}`);
		return;
	}
	console.log(
		`[build ${builds}] hash=${stats.hash} time=${Date.now() - startedAt}ms`
	);

	if (builds !== 1) return;

	const files = [...stats.compilation.fileDependencies];
	const slash = isWin ? files.filter(p => p.includes('/')) : [];
	console.log(`[deps] total=${files.length} slash=${slash.length}`);
	for (const p of files) console.log(`[dep] ${p}`);

	setTimeout(() => {
		const target = path.resolve(
			root,
			TOUCH === 'entry' ? 'src/index.js' : 'src/a.js'
		);
		fs.appendFileSync(target, `\n// touch ${Date.now()}\n`);
		console.log(`[touched] ${target}`);
	}, 2000);
});

setTimeout(() => {
	const verdict = builds >= 2 ? 'OK-REBUILD' : 'BUG-NO-REBUILD';
	console.log(`RESULT ${tag} builds=${builds} ${verdict}`);
	watching.close(() => process.exit(0));
}, 25000);
