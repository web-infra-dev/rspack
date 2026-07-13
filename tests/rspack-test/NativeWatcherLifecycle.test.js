const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const binding = require("@rspack/binding");

const ITERATIONS = 30;
const WATCHES_PER_ITERATION = 5;
const FILE_COUNT = 400;

let dir;
let files;
let dirs;

const startWatch = watcher => {
	watcher.watch(
		[files, []],
		[dirs, []],
		[[], []],
		BigInt(Date.now()),
		() => {},
		() => {}
	);
};

describe("NativeWatcher lifecycle", () => {
	beforeAll(() => {
		dir = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-native-watcher-"));
		files = [];
		dirs = [dir];
		for (let i = 0; i < FILE_COUNT; i++) {
			const sub = path.join(dir, `sub${i % 20}`);
			fs.mkdirSync(sub, { recursive: true });
			const file = path.join(sub, `f${i}.js`);
			fs.writeFileSync(file, `module.exports = ${i};`);
			files.push(file);
			if (!dirs.includes(sub)) {
				dirs.push(sub);
			}
		}
	});

	afterAll(() => {
		fs.rmSync(dir, { recursive: true, force: true });
	});

	it("does not crash when watch and close race on the same watcher", async () => {
		for (let i = 0; i < ITERATIONS; i++) {
			const watcher = new binding.NativeWatcher({ aggregateTimeout: 1 });

			for (let j = 0; j < WATCHES_PER_ITERATION; j++) {
				startWatch(watcher);
			}

			await watcher.close();

			expect(() => startWatch(watcher)).toThrow(/has been closed/);
		}
	});
});
