// Same project as repro-app.js, but drives the compiler directly so we can dump
// the exact spelling of every path handed to the watcher.
const path = require('node:path');
const fs = require('node:fs');
const { pathToFileURL } = require('node:url');

const APP = process.env.APP_DIR || path.resolve(__dirname, 'app');
const TOUCH = process.env.TOUCH_FILE || 'app/js/pages/Home/index.tsx';
const PHASE = process.env.PHASE || 'cold';
const CLEAN = process.env.CLEAN === '1';
const TOTAL_MS = Number(process.env.TOTAL_MS || 60000);

process.chdir(APP);

if (CLEAN) {
	fs.rmSync(path.join(APP, 'node_modules/.cache'), {
		recursive: true,
		force: true
	});
	console.log('[clean] removed app cache');
}

const version = require(
	path.join(APP, 'node_modules/@rspack/core/package.json')
).version;

function classify(p) {
	const fwd = p.includes('/');
	const back = p.includes('\\');
	if (fwd && back) return 'mixed';
	if (fwd) return 'slash';
	return 'native';
}

async function main() {
	const { rspack } = await import(
		pathToFileURL(path.join(APP, 'node_modules/@rspack/core/dist/index.js')).href
	);
	const base = await import(
		pathToFileURL(path.join(APP, 'tools/rspack.config.base.ts')).href
	);
	const [, configure] = await base.default('development');
	configure.cache.version = 'dev';
	configure.devtool = 'eval-cheap-module-source-map';
	configure.watchOptions = { aggregateTimeout: 256 };

	const tag = `version=${version} phase=${PHASE}`;
	const target = path.join(APP, TOUCH);
	const compiler = rspack(configure);

	let builds = 0;
	let buildsAfterTouch = 0;
	let touched = false;

	function report(label, list) {
		const files = [...list];
		const counts = { native: 0, slash: 0, mixed: 0 };
		for (const p of files) counts[classify(p)]++;
		const exact = files.includes(target);
		const norm = files.filter(
			p => path.resolve(p).toLowerCase() === path.resolve(target).toLowerCase()
		);
		console.log(
			`[${label}] total=${files.length} native=${counts.native} slash=${counts.slash} mixed=${counts.mixed} targetExact=${exact} targetSpelling=${norm[0] ? classify(norm[0]) : 'ABSENT'}`
		);
	}

	const watching = compiler.watch({}, (err, stats) => {
		builds++;
		if (touched) buildsAfterTouch++;
		if (err) {
			console.log(`[build ${builds}] ERROR ${err}`);
			return;
		}
		console.log(`[build ${builds}] hash=${stats.hash}`);
		report(`deps build ${builds}`, stats.compilation.fileDependencies);

		if (builds !== 1) return;
		console.log(`[touch target] ${target}`);
		setTimeout(() => {
			touched = true;
			fs.appendFileSync(target, `\n// touch ${Date.now()}\n`);
			console.log('[touched]');
		}, 4000);
	});

	// Wrap the watch file system so we see exactly what gets registered with watchpack.
	const wfs = compiler.watchFileSystem;
	if (wfs && typeof wfs.watch === 'function') {
		const original = wfs.watch.bind(wfs);
		let round = 0;
		wfs.watch = (files, directories, missing, ...rest) => {
			round++;
			report(`wfs.watch ${round}`, files);
			return original(files, directories, missing, ...rest);
		};
	} else {
		console.log('[wfs] not wrappable');
	}

	setTimeout(() => {
		const verdict = buildsAfterTouch >= 1 ? 'OK-REBUILD' : 'BUG-NO-REBUILD';
		console.log(
			`RESULT ${tag} builds=${builds} afterTouch=${buildsAfterTouch} ${verdict}`
		);
		watching.close(() => process.exit(0));
	}, TOTAL_MS);
}

main().catch(err => {
	console.log(`[fatal] ${err && err.stack}`);
	process.exit(0);
});
