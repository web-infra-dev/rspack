const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { NativeWatcher } = require("@rspack/binding");
const { rspack } = require("@rspack/core");

describe("NativeWatcher lifecycle", () => {
	const root = fs.mkdtempSync(path.join(os.tmpdir(), "rspack-native-lifecycle-"));
	const files = Array.from({ length: 40 }, (_, index) => {
		const file = path.join(root, `file-${index}.txt`);
		fs.writeFileSync(file, `${index}\n`);
		return file;
	});
	const raceDirectories = Array.from({ length: 20 }, (_, index) => {
		const directory = path.join(root, `sub-${index}`);
		fs.mkdirSync(directory, { recursive: true });
		return directory;
	});
	const raceFiles = Array.from({ length: 400 }, (_, index) => {
		const file = path.join(
			raceDirectories[index % raceDirectories.length],
			`file-${index}.txt`,
		);
		fs.writeFileSync(file, `${index}\n`);
		return file;
	});
	const startWatch = (watcher, watchedFiles, directories) =>
		watcher.watch(
			[watchedFiles, []],
			[directories, []],
			[[], []],
			BigInt(Date.now()),
			() => {},
			() => {},
		);

	afterAll(() => {
		fs.rmSync(root, { recursive: true, force: true });
	});

	it("serializes immediate and repeated close with watch and pause", async () => {
		for (let index = 0; index < 1000; index++) {
			const watcher = new NativeWatcher({ aggregateTimeout: 0 });
			const watch = () => startWatch(watcher, files, [root]);

			watch();
			watch();
			watcher.pause();
			await Promise.all([watcher.close(), watcher.close()]);

			expect(watch).toThrow("The native watcher has been closed");
		}
	});

	it("serializes immediate close with overlapping watches on a large dependency set", async () => {
		for (let index = 0; index < 30; index++) {
			const watcher = new NativeWatcher({ aggregateTimeout: 1 });
			const watch = () =>
				startWatch(watcher, raceFiles, [root, ...raceDirectories]);

			for (let count = 0; count < 5; count++) {
				watch();
			}
			await watcher.close();

			expect(watch).toThrow("The native watcher has been closed");
		}
	});

	it("recreates a closed native watcher with its complete dependency set", async () => {
		const compiler = rspack({
			context: root,
			entry: files[0],
			experiments: { nativeWatcher: true },
		});
		const watchFileSystem = compiler.watchFileSystem;
		const dependencies = values =>
			Object.assign(values, { added: [], removed: [] });
		const watch = callback =>
			watchFileSystem.watch(
				dependencies(files),
				dependencies([root]),
				dependencies([]),
				Date.now(),
				{ aggregateTimeout: 0 },
				callback,
				() => {},
			);

		const firstNativeWatcher = watchFileSystem.getNativeWatcher({});
		const first = watch(() => {});
		first.close();

		const secondNativeWatcher = watchFileSystem.getNativeWatcher({});
		expect(secondNativeWatcher).not.toBe(firstNativeWatcher);

		let second;
		try {
			const changes = await new Promise((resolve, reject) => {
				const timeout = setTimeout(
					() => reject(new Error("timed out waiting for recreated native watcher")),
					5000,
				);
				second = watch((_error, _fileTimes, _contextTimes, changes) => {
					clearTimeout(timeout);
					resolve(changes);
				});
				fs.writeFileSync(files[0], "changed\n");
				const changedAt = new Date(Date.now() + 2000);
				fs.utimesSync(files[0], changedAt, changedAt);
				watchFileSystem.triggerEvent("change", files[0]);
			});

			expect([...changes]).toContain(files[0]);
		} finally {
			second?.close();
			await Promise.all([
				firstNativeWatcher.close(),
				secondNativeWatcher.close(),
			]);
		}
	});
});
