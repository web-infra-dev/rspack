// Drives the reporter's real project (nuintun/rspack-antd-builder) for #15352:
// boots its dev server, waits for the first successful compile, touches a page
// file and reports whether a second compile happens.
const { spawn } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const APP = process.env.APP_DIR || path.resolve(__dirname, 'app');
const TOUCH = process.env.TOUCH_FILE || 'app/js/pages/Home/index.tsx';
const PHASE = process.env.PHASE || 'cold';
const CLEAN = process.env.CLEAN === '1';
const TOTAL_MS = Number(process.env.TOTAL_MS || 60000);

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
const tag = `version=${version} phase=${PHASE}`;

const child = spawn(process.execPath, ['tools/rspack.config.dev.ts'], {
	cwd: APP,
	stdio: ['ignore', 'pipe', 'pipe']
});

let compiles = 0;
let touched = false;

function feed(buf) {
	const text = buf.toString();
	process.stdout.write(text);
	const hits = text.match(/compiled/gi);
	if (!hits) return;
	compiles += hits.length;
	console.log(`[compiles] ${compiles}`);
	if (touched) return;
	touched = true;
	setTimeout(() => {
		const target = path.join(APP, TOUCH);
		fs.appendFileSync(target, `\n// touch ${Date.now()}\n`);
		console.log(`[touched] ${target}`);
	}, 3000);
}

child.stdout.on('data', feed);
child.stderr.on('data', feed);

setTimeout(() => {
	const verdict = compiles >= 2 ? 'OK-REBUILD' : 'BUG-NO-REBUILD';
	console.log(`RESULT ${tag} compiles=${compiles} ${verdict}`);
	child.kill('SIGKILL');
	setTimeout(() => process.exit(0), 500);
}, TOTAL_MS);
