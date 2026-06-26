import fs from "fs";
import { createDeferred, createRemoteServer } from "./remoteServer";

const remoteServer = createRemoteServer();

function getFederation() {
	const federation = __webpack_require__.federation;
	expect(federation).toBeTruthy();
	return federation;
}

function getFederationInstance() {
	const instance = getFederation().instance;
	expect(instance).toBeTruthy();
	expect(typeof instance.removeRemote).toBe("function");
	expect(typeof instance.registerRemotes).toBe("function");
	return instance;
}

async function removeRemoteA() {
	const instance = getFederationInstance();
	const remote = instance.options.remotes.find(
		remote => remote.name === "remoteA" || remote.alias === "remoteA"
	);
	const removePromise = instance.removeRemote("remoteA");
	expect(typeof removePromise?.then).toBe("function");
	await removePromise;
	if (remote) {
		instance.registerRemotes([{ ...remote }]);
	}
}

async function renderPageA() {
	const { render } = await import("./pageA");
	return render();
}

function getChunkCacheControl() {
	const control = __webpack_require__.chunkCacheControls?.readFileVm;
	expect(control).toBeTruthy();
	return control;
}

function getRemoteChunkIds() {
	return __webpack_require__.remotesLoadingData?.remoteKeyToChunkIds?.remoteA || [];
}

function blockNextAsyncChunkRead() {
	const deferred = createDeferred();
	const originalReadFile = fs.readFile;
	let blockedCall;

	fs.readFile = function (filename, ...args) {
		if (!blockedCall && /\.js$/.test(String(filename))) {
			blockedCall = { filename, args };
			deferred.resolve();
			return;
		}
		return originalReadFile.call(this, filename, ...args);
	};

	return {
		wait() {
			return deferred.promise;
		},
		release() {
			if (blockedCall) {
				originalReadFile.call(fs, blockedCall.filename, ...blockedCall.args);
			}
			fs.readFile = originalReadFile;
		},
		restore() {
			fs.readFile = originalReadFile;
		}
	};
}

it("should wait pending readFileVm remote chunk before clearing it", async () => {
	const chunkControl = getChunkCacheControl();
	const clearCalls = [];
	const originalClear = chunkControl.clear;
	chunkControl.clear = chunkIds => {
		const result = originalClear(chunkIds);
		clearCalls.push({ chunkIds, result });
		return result;
	};
	try {
		const remoteChunkIds = getRemoteChunkIds();
		expect(remoteChunkIds.length).toBeGreaterThan(0);

		const blockedRead = blockNextAsyncChunkRead();
		const oldRequest = renderPageA();
		await blockedRead.wait();

		const pendingRemoteChunkIds = remoteChunkIds.filter(chunkId =>
			Array.isArray(chunkControl.getState(chunkId))
		);
		expect(pendingRemoteChunkIds.length).toBeGreaterThan(0);
		const previousGenerations = pendingRemoteChunkIds.map(chunkId =>
			chunkControl.getGeneration(chunkId)
		);
		const beforeClear = remoteServer.snapshot();

		remoteServer.setVersion("v2");
		const removePromise = removeRemoteA();
		await Promise.resolve();
		expect(remoteServer.snapshot()).toEqual(beforeClear);

		blockedRead.release();
		await expect(oldRequest).resolves.toBe("pageA:./A:v1");
		await removePromise;

		expect(clearCalls).toEqual([
			{ chunkIds: pendingRemoteChunkIds, result: pendingRemoteChunkIds }
		]);
		for (const chunkId of pendingRemoteChunkIds) {
			expect(chunkControl.getState(chunkId)).toBeUndefined();
		}
		expect(
			pendingRemoteChunkIds.map(chunkId => chunkControl.getGeneration(chunkId))
		).toEqual(previousGenerations.map(generation => generation + 1));
		expect(await renderPageA()).toBe("pageA:./A:v2");
	} finally {
		chunkControl.clear = originalClear;
	}
});

it("should drop stale readFileVm chunk results after timeout", async () => {
	const timeoutKey = "__rspack_clear_cache_wait_timeout__";
	const hadTimeout = Object.prototype.hasOwnProperty.call(globalThis, timeoutKey);
	const previousTimeout = globalThis[timeoutKey];
	globalThis[timeoutKey] = 0;

	try {
		remoteServer.setVersion("v3");
		await removeRemoteA();

		const chunkControl = getChunkCacheControl();
		const remoteChunkIds = getRemoteChunkIds();
		const blockedRead = blockNextAsyncChunkRead();
		const oldRequest = renderPageA();
		await blockedRead.wait();

		const pendingRemoteChunkIds = remoteChunkIds.filter(chunkId =>
			Array.isArray(chunkControl.getState(chunkId))
		);
		expect(pendingRemoteChunkIds.length).toBeGreaterThan(0);
		const previousGenerations = pendingRemoteChunkIds.map(chunkId =>
			chunkControl.getGeneration(chunkId)
		);

		remoteServer.setVersion("v4");
		await removeRemoteA();

		for (const chunkId of pendingRemoteChunkIds) {
			expect(chunkControl.getState(chunkId)).toBeUndefined();
		}
		expect(
			pendingRemoteChunkIds.map(chunkId => chunkControl.getGeneration(chunkId))
		).toEqual(previousGenerations.map(generation => generation + 1));

		blockedRead.release();
		await expect(oldRequest).rejects.toThrow("stale");
		for (const chunkId of pendingRemoteChunkIds) {
			expect(chunkControl.getState(chunkId)).toBeUndefined();
		}
		expect(await renderPageA()).toBe("pageA:./A:v4");
	} finally {
		if (hadTimeout) {
			globalThis[timeoutKey] = previousTimeout;
		} else {
			delete globalThis[timeoutKey];
		}
	}
});
