const { rspack } = require("@rspack/core");

class FakeNativeWatcher {
	acknowledgements = [];
	pauses = 0;
	pendingDrain = {
		changedFiles: [],
		removedFiles: [],
		generation: 0
	};

	watch(_files, _directories, _missing, _startTime, onAggregate) {
		this.onAggregate = onAggregate;
	}

	takePendingEvents() {
		return this.pendingDrain;
	}

	acknowledgePendingEvents(generation) {
		this.acknowledgements.push(generation);
	}

	pause() {
		this.pauses++;
	}

	close() {
		return Promise.resolve();
	}
}

const dependencies = () =>
	Object.assign([], {
		added: [],
		removed: []
	});

function createWatcherHarness() {
	const nativeWatcher = new FakeNativeWatcher();
	const purged = [];
	const callbackChanges = [];
	const compiler = rspack({
		context: __dirname,
		entry: __filename,
		experiments: { nativeWatcher: true }
	});
	compiler.inputFileSystem.purge = path => purged.push(path);
	const watchFileSystem = compiler.watchFileSystem;
	watchFileSystem.getNativeWatcher = () => nativeWatcher;

	const watcher = watchFileSystem.watch(
		dependencies(),
		dependencies(),
		dependencies(),
		Date.now(),
		{},
		(_error, _fileTimes, _contextTimes, changes) =>
			callbackChanges.push(changes),
		() => {}
	);

	return { nativeWatcher, purged, callbackChanges, watcher };
}

describe("NativeWatchFileSystem aggregate generations", () => {
	it("suppresses a callback superseded by a synchronous native drain", () => {
		const { nativeWatcher, purged, callbackChanges, watcher } =
			createWatcherHarness();

		nativeWatcher.pendingDrain = {
			changedFiles: ["/changed"],
			removedFiles: [],
			generation: 2
		};
		expect(watcher.getInfo().changes).toEqual(new Set(["/changed"]));
		expect(purged).toEqual(["/changed"]);

		nativeWatcher.onAggregate(null, {
			changedFiles: ["/changed"],
			removedFiles: [],
			generation: 1
		});
		expect(nativeWatcher.acknowledgements).toEqual([1]);
		expect(nativeWatcher.pauses).toBe(0);
		expect(purged).toEqual(["/changed"]);
		expect(callbackChanges).toEqual([]);

		nativeWatcher.onAggregate(null, {
			changedFiles: ["/next"],
			removedFiles: [],
			generation: 3
		});
		expect(nativeWatcher.acknowledgements).toEqual([1, 3]);
		expect(nativeWatcher.pauses).toBe(1);
		expect(purged).toEqual(["/changed", "/next"]);
		expect(callbackChanges).toEqual([new Set(["/next"])]);
	});

	it("accepts a newer aggregate after the native generation wraps", () => {
		const { nativeWatcher, purged, callbackChanges, watcher } =
			createWatcherHarness();

		nativeWatcher.pendingDrain = {
			changedFiles: ["/drained"],
			removedFiles: [],
			generation: 0xffffffff
		};
		expect(watcher.getInfo().changes).toEqual(new Set(["/drained"]));

		nativeWatcher.onAggregate(null, {
			changedFiles: ["/stale"],
			removedFiles: [],
			generation: 0xfffffffe
		});
		nativeWatcher.onAggregate(null, {
			changedFiles: ["/wrapped"],
			removedFiles: [],
			generation: 0
		});

		expect(nativeWatcher.acknowledgements).toEqual([0xfffffffe, 0]);
		expect(nativeWatcher.pauses).toBe(1);
		expect(purged).toEqual(["/drained", "/wrapped"]);
		expect(callbackChanges).toEqual([new Set(["/wrapped"])]);
	});
});
