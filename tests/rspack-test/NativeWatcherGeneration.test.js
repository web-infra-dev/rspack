const { rspack } = require("@rspack/core");

class FakeNativeWatcher {
	acknowledgements = [];
	pauses = 0;
	closes = 0;
	drains = 0;
	triggered = [];
	pendingDrain = {
		changedFiles: [],
		removedFiles: [],
		generation: 0
	};

	watch(_files, _directories, _missing, _startTime, onAggregate, onRaw) {
		this.onAggregate = onAggregate;
		this.onRaw = onRaw;
	}

	takePendingEvents() {
		this.drains++;
		return this.pendingDrain;
	}

	acknowledgePendingEvents(generation) {
		this.acknowledgements.push(generation);
	}

	pause() {
		this.pauses++;
	}

	triggerEvent(kind, path) {
		this.triggered.push([kind, path]);
	}

	close() {
		this.closes++;
		return Promise.resolve();
	}
}

const dependencies = () =>
	Object.assign([], {
		added: [],
		removed: []
	});

function createWatcherHarness(callbackUndelayed = () => {}) {
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
		callbackUndelayed
	);

	return { nativeWatcher, purged, callbackChanges, watcher, watchFileSystem };
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

	it("ignores a raw callback retained by an earlier watch generation", () => {
		const stale = [];
		const fresh = [];
		const { nativeWatcher, watchFileSystem } = createWatcherHarness(path =>
			stale.push(path)
		);
		const staleRaw = nativeWatcher.onRaw;

		const currentWatcher = watchFileSystem.watch(
			dependencies(),
			dependencies(),
			dependencies(),
			Date.now(),
			{},
			() => {},
			path => fresh.push(path)
		);

		staleRaw({ kind: "change", path: "/stale" });
		nativeWatcher.onRaw({ kind: "change", path: "/fresh" });

		expect(stale).toEqual([]);
		expect(fresh).toEqual(["/fresh"]);

		currentWatcher.close();
		nativeWatcher.onRaw({ kind: "change", path: "/after-close" });
		expect(fresh).toEqual(["/fresh"]);
	});

	it("forwards concrete child events to long-lived watch-file-system listeners", () => {
		const events = [];
		const { nativeWatcher, watchFileSystem } = createWatcherHarness();
		watchFileSystem.on("change", (file, mtime) =>
			events.push(["change", file, mtime])
		);
		watchFileSystem.on("remove", file => events.push(["remove", file]));
		const staleRaw = nativeWatcher.onRaw;

		const currentWatcher = watchFileSystem.watch(
			dependencies(),
			dependencies(),
			dependencies(),
			Date.now(),
			{},
			() => {},
			() => {}
		);

		staleRaw({ kind: "change", path: "/src/stale.js" });
		staleRaw({ kind: "remove", path: "/src/stale.js" });
		nativeWatcher.onRaw({ kind: "change", path: "/src/created.js" });
		nativeWatcher.onRaw({ kind: "remove", path: "/src/created.js" });
		expect(events).toEqual([
			["change", "/src/created.js", expect.any(Number)],
			["remove", "/src/created.js"]
		]);

		currentWatcher.close();
		nativeWatcher.onRaw({ kind: "change", path: "/src/after-close.js" });
		nativeWatcher.onRaw({ kind: "remove", path: "/src/after-close.js" });
		expect(events).toHaveLength(2);
	});

	it("prevents stale watcher handles and callbacks from affecting the live generation", () => {
		const events = [];
		const {
			nativeWatcher,
			purged,
			callbackChanges,
			watcher: staleWatcher,
			watchFileSystem
		} = createWatcherHarness();
		watchFileSystem.on("aggregated", (changes, removals) =>
			events.push([changes, removals])
		);
		const staleAggregate = nativeWatcher.onAggregate;
		const staleShim = watchFileSystem.watcher;

		const currentChanges = [];
		const currentWatcher = watchFileSystem.watch(
			dependencies(),
			dependencies(),
			dependencies(),
			Date.now(),
			{},
			(_error, _fileTimes, _contextTimes, changes) =>
				currentChanges.push(changes),
			() => {}
		);

		nativeWatcher.pendingDrain = {
			changedFiles: ["/src/live.js"],
			removedFiles: [],
			generation: 2
		};
		staleWatcher.pause();
		expect(staleWatcher.getInfo()).toEqual({
			changes: new Set(),
			removals: new Set(),
			fileTimeInfoEntries: new Map(),
			contextTimeInfoEntries: new Map()
		});
		staleWatcher.close();
		staleShim._onChange("/src/stale.js");
		staleShim._onRemove("/src/stale.js");
		staleAggregate(null, {
			changedFiles: ["/src/stale.js"],
			removedFiles: [],
			generation: 1
		});

		expect(nativeWatcher.pauses).toBe(0);
		expect(nativeWatcher.drains).toBe(0);
		expect(nativeWatcher.closes).toBe(0);
		expect(nativeWatcher.triggered).toEqual([]);
		expect(nativeWatcher.acknowledgements).toEqual([1]);
		expect(purged).toEqual([]);
		expect(callbackChanges).toEqual([]);
		expect(currentChanges).toEqual([]);
		expect(events).toEqual([]);

		nativeWatcher.onAggregate(null, {
			changedFiles: ["/src/live.js"],
			removedFiles: [],
			generation: 2
		});
		expect(nativeWatcher.pauses).toBe(1);
		expect(nativeWatcher.acknowledgements).toEqual([1, 2]);
		expect(purged).toEqual(["/src/live.js"]);
		expect(callbackChanges).toEqual([]);
		expect(currentChanges).toEqual([new Set(["/src/live.js"])]);
		expect(events).toEqual([[new Set(["/src/live.js"]), new Set()]]);
		watchFileSystem.watcher._onChange("/src/live.js");
		watchFileSystem.watcher._onRemove("/src/live.js");
		expect(nativeWatcher.triggered).toEqual([
			["change", "/src/live.js"],
			["remove", "/src/live.js"]
		]);

		currentWatcher.close();
		expect(nativeWatcher.closes).toBe(1);
	});
});
