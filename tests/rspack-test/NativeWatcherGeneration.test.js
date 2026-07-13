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

describe("NativeWatchFileSystem aggregate generations", () => {
	it("suppresses a callback superseded by a synchronous native drain", () => {
		const nativeWatcher = new FakeNativeWatcher();
		const purged = [];
		const callbackCalls = [];
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
			(...args) => callbackCalls.push(args),
			() => {}
		);

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
		expect(callbackCalls).toEqual([]);

		nativeWatcher.onAggregate(null, {
			changedFiles: ["/next"],
			removedFiles: [],
			generation: 3
		});
		expect(nativeWatcher.acknowledgements).toEqual([1, 3]);
		expect(nativeWatcher.pauses).toBe(1);
		expect(purged).toEqual(["/changed", "/next"]);
		expect(callbackCalls).toHaveLength(1);
		expect(callbackCalls[0][3]).toEqual(new Set(["/next"]));
	});
});
