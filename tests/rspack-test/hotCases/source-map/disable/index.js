import path from 'path';
import fs from 'fs';
import value from './file'

let prevFullhash = __webpack_hash__;

function check() {
	const map = path.join(__dirname, `main.${prevFullhash}.hot-update.js.map`)
	expect(fs.existsSync(map)).toBe(false);
	prevFullhash = __webpack_hash__;
}

it("should not have hot-update.map file when hmr", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	check();
	await NEXT_HMR();
	check();
	await NEXT_HMR();
	check();
});

module.hot.accept("./file");
