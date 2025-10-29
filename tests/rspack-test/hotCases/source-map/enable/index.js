import path from 'path';
import fs from 'fs';
import value from './file'

let prevFullhash = __webpack_hash__;

function check() {
	const map = path.join(__dirname, `main.${prevFullhash}.hot-update.js.map`)
	expect(fs.existsSync(map)).toBe(true);
	prevFullhash = __webpack_hash__;
}

it("should have hot-update.map file when hmr", async () => {
	const done = err => (err ? reject(err) : resolve());
	expect(value).toBe(1);
	await NEXT_HMR();
	check();
	await NEXT_HMR();
	check();
	await NEXT_HMR();
	check();
});

module.hot.accept("./file");
