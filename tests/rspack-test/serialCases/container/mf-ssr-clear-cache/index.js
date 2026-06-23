import { createRemoteServer } from "./remoteServer";

const remoteServer = createRemoteServer();
let remoteATemplate;

function withoutRemoteEntryClears(snapshot) {
	const { remoteEntryClears, ...rest } = snapshot;
	return rest;
}

function expectSnapshotUnchangedExceptRemoteEntryClears(snapshot, before) {
	expect(withoutRemoteEntryClears(snapshot)).toEqual(
		withoutRemoteEntryClears(before)
	);
}

function createDeferred() {
	let resolve;
	let reject;
	const promise = new Promise((r, j) => {
		resolve = r;
		reject = j;
	});
	return { promise, resolve, reject };
}

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

function getRegisteredRemoteA(instance = getFederationInstance()) {
	const remote = instance.options.remotes.find(
		remote => remote.name === "remoteA" || remote.alias === "remoteA"
	);
	expect(remote).toBeTruthy();
	const {
		name,
		alias,
		entry,
		type,
		shareScope,
		entryGlobalName,
		version
	} = remote;
	return Object.fromEntries(
		Object.entries({
			name,
			alias,
			entry,
			type,
			shareScope,
			entryGlobalName,
			version
		}).filter(([, value]) => value !== undefined)
	);
}

function getRemoteATemplate(instance = getFederationInstance()) {
	if (!remoteATemplate) {
		remoteATemplate = getRegisteredRemoteA(instance);
	}
	return remoteATemplate;
}

async function removeRemoteA(instance = getFederationInstance()) {
	const removePromise = instance.removeRemote("remoteA");
	expect(typeof removePromise?.then).toBe("function");
	await removePromise;
}

async function registerRemoteA(remote, instance = getFederationInstance()) {
	const registerPromise = instance.registerRemotes([remote]);
	if (registerPromise) {
		expect(typeof registerPromise.then).toBe("function");
		await registerPromise;
	}
}

function createRemoteAForVersion(version, instance = getFederationInstance()) {
	return {
		...getRemoteATemplate(instance),
		entry: `http://localhost:3001/remote-project-${version}/remoteEntry.js`
	};
}

async function clearAndRegisterRemoteA(version) {
	const instance = getFederationInstance();
	const remote = createRemoteAForVersion(version, instance);
	await removeRemoteA(instance);
	await registerRemoteA(remote, instance);
}

async function renderPageA() {
	const { render } = await import("./pageA");
	return render();
}

async function renderPageB() {
	const { render } = await import("./pageB");
	return render();
}

async function renderBothPages() {
	return {
		pageA: await renderPageA(),
		pageB: await renderPageB()
	};
}

async function withBrowserEnvironment(fn) {
	const hadWindow = Object.prototype.hasOwnProperty.call(globalThis, "window");
	const hadDocument = Object.prototype.hasOwnProperty.call(
		globalThis,
		"document"
	);
	const previousWindow = globalThis.window;
	const previousDocument = globalThis.document;
	globalThis.window = globalThis;
	globalThis.document = {};
	try {
		return await fn();
	} finally {
		if (hadWindow) {
			globalThis.window = previousWindow;
		} else {
			delete globalThis.window;
		}
		if (hadDocument) {
			globalThis.document = previousDocument;
		} else {
			delete globalThis.document;
		}
	}
}

function getNodeChunkCacheControl() {
	const controls = __webpack_require__.chunkCacheControls || {};
	return controls.require || controls.readFileVm;
}

function getRemoteChunkIds(name = "remoteA") {
	return __webpack_require__.remotesLoadingData?.remoteKeyToChunkIds?.[name] || [];
}

function getLoadedRemoteChunkIds(name = "remoteA") {
	const control = getNodeChunkCacheControl();
	expect(control).toBeTruthy();
	expect(typeof control.getState).toBe("function");
	return getRemoteChunkIds(name).filter(chunkId => {
		const state = control.getState(chunkId);
		return state === 0 || state === 1;
	});
}

function getRemoteChunkGenerations(chunkIds) {
	const control = getNodeChunkCacheControl();
	expect(typeof control.getGeneration).toBe("function");
	return chunkIds.map(chunkId => control.getGeneration(chunkId));
}

function getChunkRequireCachePath(chunkId) {
	return __non_webpack_require__("node:path").resolve(
		__dirname,
		__webpack_require__.u(chunkId)
	);
}

function getNodeRequireCache() {
	return __non_webpack_require__("node:module")._cache;
}

function getTrackedModuleIds(name = "remoteA") {
	const data = __webpack_require__.remotesLoadingData;
	const moduleIds = new Set([
		...(data.remoteKeyToRemoteModuleIds?.[name] || []),
		...(data.remoteKeyToExternalModuleIds?.[name] || [])
	]);
	const queue = [...(data.remoteKeyToRemoteModuleIds?.[name] || [])];
	for (let i = 0; i < queue.length; i++) {
		const moduleId = queue[i];
		for (const consumerId of data.remoteModuleIdToConsumerModuleIds?.[
			moduleId
		] || []) {
			if (!moduleIds.has(consumerId)) {
				moduleIds.add(consumerId);
				queue.push(consumerId);
			}
		}
		for (const parentId of data.consumerModuleIdToParentModuleIds?.[moduleId] ||
			[]) {
			if (!moduleIds.has(parentId)) {
				moduleIds.add(parentId);
				queue.push(parentId);
			}
		}
	}
	return [...moduleIds];
}

function getTrackedModuleCacheSize() {
	return getTrackedModuleIds().filter(moduleId => __webpack_require__.c[moduleId])
		.length;
}

it("should invalidate SSR remote and affected consumer caches without preloading", async () => {
	expect(await renderBothPages()).toEqual({
		pageA: "pageA:./A:v1",
		pageB: "pageB:./B:v1"
	});
	expect(remoteServer.routeExecutions).toEqual({
		pageA: 1,
		pageB: 1
	});
	expect(remoteServer.largeRemotePayloads).toEqual([
		expect.objectContaining({ expose: "./A", version: "v1", size: 50000 }),
		expect.objectContaining({ expose: "./B", version: "v1", size: 50000 })
	]);
	const loadedRemoteChunkIds = getLoadedRemoteChunkIds();
	expect(loadedRemoteChunkIds.length).toBeGreaterThan(0);
	expect(getTrackedModuleCacheSize()).toBeGreaterThan(0);
	const loadedRemoteChunkGenerations =
		getRemoteChunkGenerations(loadedRemoteChunkIds);
	const loadedRemoteChunkCachePaths = loadedRemoteChunkIds.map(chunkId =>
		getChunkRequireCachePath(chunkId)
	);
	const nodeRequireCache = getNodeRequireCache();
	for (const chunkCachePath of loadedRemoteChunkCachePaths) {
		nodeRequireCache[chunkCachePath] = {
			exports: { __rspackClearCacheTestMarker: true }
		};
		expect(nodeRequireCache[chunkCachePath]).toBeTruthy();
	}

	const beforeClear = remoteServer.snapshot();
	const remoteA = createRemoteAForVersion("v2");
	await removeRemoteA();

	const afterRemove = remoteServer.snapshot();
	expectSnapshotUnchangedExceptRemoteEntryClears(afterRemove, beforeClear);
	expect(getTrackedModuleCacheSize()).toBe(0);
	await registerRemoteA(remoteA);
	expectSnapshotUnchangedExceptRemoteEntryClears(
		remoteServer.snapshot(),
		beforeClear
	);
	const chunkControl = getNodeChunkCacheControl();
	for (const chunkId of loadedRemoteChunkIds) {
		expect(chunkControl.getState(chunkId)).toBeUndefined();
	}
	for (const chunkCachePath of loadedRemoteChunkCachePaths) {
		expect(nodeRequireCache[chunkCachePath]).toBeUndefined();
	}
	expect(getRemoteChunkGenerations(loadedRemoteChunkIds)).toEqual(
		loadedRemoteChunkGenerations.map(generation => generation + 1)
	);

	expect(await renderBothPages()).toEqual({
		pageA: "pageA:./A:v2",
		pageB: "pageB:./B:v2"
	});
	expect(remoteServer.routeExecutions).toEqual({
		pageA: 2,
		pageB: 2
	});
	expect(remoteServer.largeRemotePayloads).toEqual([
		expect.objectContaining({ expose: "./A", version: "v1", size: 50000 }),
		expect.objectContaining({ expose: "./B", version: "v1", size: 50000 }),
		expect.objectContaining({ expose: "./A", version: "v2", size: 50000 }),
		expect.objectContaining({ expose: "./B", version: "v2", size: 50000 })
	]);
	expect(remoteServer.remoteEntryLoads.length).toBeGreaterThan(
		beforeClear.remoteEntryLoads
	);
	expect(remoteServer.remoteGets.length).toBeGreaterThan(beforeClear.remoteGets);

	const beforeSecondClear = remoteServer.snapshot();
	await clearAndRegisterRemoteA("v3");
	expectSnapshotUnchangedExceptRemoteEntryClears(
		remoteServer.snapshot(),
		beforeSecondClear
	);
	expect(getTrackedModuleCacheSize()).toBe(0);
	expect(await renderBothPages()).toEqual({
		pageA: "pageA:./A:v3",
		pageB: "pageB:./B:v3"
	});
	expect(remoteServer.routeExecutions).toEqual({
		pageA: 3,
		pageB: 3
	});
	expect(remoteServer.largeRemotePayloads).toEqual([
		expect.objectContaining({ expose: "./A", version: "v1", size: 50000 }),
		expect.objectContaining({ expose: "./B", version: "v1", size: 50000 }),
		expect.objectContaining({ expose: "./A", version: "v2", size: 50000 }),
		expect.objectContaining({ expose: "./B", version: "v2", size: 50000 }),
		expect.objectContaining({ expose: "./A", version: "v3", size: 50000 }),
		expect.objectContaining({ expose: "./B", version: "v3", size: 50000 })
	]);
	expect(remoteServer.remoteEntryLoads).toEqual(
		expect.arrayContaining([
			expect.stringContaining(
				"v1:http://localhost:3001/remote-project-v1/remoteEntry.js"
			),
			expect.stringContaining(
				"v2:http://localhost:3001/remote-project-v2/remoteEntry.js"
			),
			expect.stringContaining(
				"v3:http://localhost:3001/remote-project-v3/remoteEntry.js"
			)
		])
	);
});

it("should reject and keep old caches usable when clear fails", async () => {
	await clearAndRegisterRemoteA("v3");
	expect(await renderPageA()).toBe("pageA:./A:v3");

	const loadedRemoteChunkIds = getLoadedRemoteChunkIds();
	const loadedRemoteChunkGenerations =
		getRemoteChunkGenerations(loadedRemoteChunkIds);
	const beforeClear = remoteServer.snapshot();
	const control = getNodeChunkCacheControl();
	const loadedRemoteChunkStates = loadedRemoteChunkIds.map(chunkId =>
		control.getState(chunkId)
	);
	const originalClear = control.clear;
	control.clear = chunkIds => {
		originalClear(chunkIds);
		throw new Error("chunk cache clear failed");
	};

	const originalWarn = console.warn;
	const originalError = console.error;
	console.warn = (...args) => {
		if (!String(args[0]).includes("removeRemote failed")) {
			originalWarn(...args);
		}
	};
	console.error = (...args) => {
		if (!String(args[0]).includes("removeRemote failed")) {
			originalError(...args);
		}
	};
	try {
		await expect(removeRemoteA()).rejects.toThrow("chunk cache clear failed");
	} finally {
		control.clear = originalClear;
		console.warn = originalWarn;
		console.error = originalError;
	}

	expectSnapshotUnchangedExceptRemoteEntryClears(
		remoteServer.snapshot(),
		beforeClear
	);
	expect(loadedRemoteChunkIds.map(chunkId => control.getState(chunkId))).toEqual(
		loadedRemoteChunkStates
	);
	expect(getRemoteChunkGenerations(loadedRemoteChunkIds)).toEqual(
		loadedRemoteChunkGenerations
	);
	expect(await renderPageA()).toBe("pageA:./A:v3");

	await clearAndRegisterRemoteA("v4");
	expect(await renderPageA()).toBe("pageA:./A:v4");
});

it("should prevent pending old remote load from updating future caches", async () => {
	await clearAndRegisterRemoteA("v2");

	remoteServer.blockNextRemoteGet();
	const oldRequest = renderPageA();
	await remoteServer.waitForBlockedRemoteGet();

	const beforeClear = remoteServer.snapshot();
	const clearPromise = clearAndRegisterRemoteA("v3");

	expectSnapshotUnchangedExceptRemoteEntryClears(
		remoteServer.snapshot(),
		beforeClear
	);
	await Promise.resolve();
	expectSnapshotUnchangedExceptRemoteEntryClears(
		remoteServer.snapshot(),
		beforeClear
	);

	remoteServer.resolveBlockedRemoteGet();
	await expect(oldRequest).resolves.toBe("pageA:./A:v2");
	await clearPromise;

	expect(getTrackedModuleCacheSize()).toBe(0);
	await expect(renderPageB()).resolves.toBe("pageB:./B:v3");
	expect(await renderPageA()).toBe("pageA:./A:v3");
});

it("should keep pending old remote load errors scoped to the old request", async () => {
	await clearAndRegisterRemoteA("v2");

	remoteServer.blockNextRemoteGet();
	const oldRequest = renderPageA();
	await remoteServer.waitForBlockedRemoteGet();

	const beforeClear = remoteServer.snapshot();
	const clearPromise = clearAndRegisterRemoteA("v3");

	expectSnapshotUnchangedExceptRemoteEntryClears(
		remoteServer.snapshot(),
		beforeClear
	);
	await Promise.resolve();
	expectSnapshotUnchangedExceptRemoteEntryClears(
		remoteServer.snapshot(),
		beforeClear
	);

	remoteServer.rejectBlockedRemoteGet(new Error("remoteA v2 failed"));
	await expect(oldRequest).rejects.toThrow("remoteA v2 failed");
	await clearPromise;

	expect(getTrackedModuleCacheSize()).toBe(0);
	await expect(renderPageB()).resolves.toBe("pageB:./B:v3");
	expect(await renderPageA()).toBe("pageA:./A:v3");
});

it("should continue clear after pending old remote load timeout", async () => {
	const timeoutKey = "__rspack_clear_cache_wait_timeout__";
	const hadTimeout = Object.prototype.hasOwnProperty.call(globalThis, timeoutKey);
	const previousTimeout = globalThis[timeoutKey];
	globalThis[timeoutKey] = 0;

	try {
		await clearAndRegisterRemoteA("v2");

		remoteServer.blockNextRemoteGet();
		const oldRequest = renderPageA();
		await remoteServer.waitForBlockedRemoteGet();

		const beforeClear = remoteServer.snapshot();
		await clearAndRegisterRemoteA("v3");

		expectSnapshotUnchangedExceptRemoteEntryClears(
			remoteServer.snapshot(),
			beforeClear
		);
		await expect(renderPageB()).resolves.toBe("pageB:./B:v3");

		remoteServer.resolveBlockedRemoteGet();
		await expect(oldRequest).resolves.toBe("pageA:./A:v2");
		expect(await renderPageA()).toBe("pageA:./A:v3");
	} finally {
		if (hadTimeout) {
			globalThis[timeoutKey] = previousTimeout;
		} else {
			delete globalThis[timeoutKey];
		}
	}
});

it("should avoid broad consumer cache cleanup in browser runtime", async () => {
	await clearAndRegisterRemoteA("v2");
	expect(await renderPageA()).toBe("pageA:./A:v2");

	const beforeClear = remoteServer.snapshot();
	await withBrowserEnvironment(() => clearAndRegisterRemoteA("v3"));

	expectSnapshotUnchangedExceptRemoteEntryClears(
		remoteServer.snapshot(),
		beforeClear
	);
	expect(await renderPageA()).toBe("pageA:./A:v2");
	expect(remoteServer.snapshot().routeExecutions.pageA).toBe(
		beforeClear.routeExecutions.pageA
	);
});

it("should remove unloaded and loading shared records from the target remote", async () => {
	const shareMap = globalThis.__FEDERATION__.__SHARE__;
	const instanceId = "__rspack_clear_cache_shared_test__";
	const previous = shareMap[instanceId];
	const loadingDeferred = createDeferred();
	const loadingShared = {
		from: "remoteA",
		useIn: []
	};
	const staleLoading = loadingDeferred.promise.then(factory => {
		loadingShared.loaded = true;
		loadingShared.lib = factory;
		return factory;
	});
	loadingShared.loading = staleLoading;
	const loadedShared = {
		from: "remoteA",
		loaded: true,
		lib: () => "loaded",
		useIn: ["host"]
	};
	const otherRemoteShared = {
		from: "remoteB",
		get: () => Promise.resolve(() => "other"),
		useIn: []
	};

	shareMap[instanceId] = {
		default: {
			unloaded: {
				"1.0.0": {
					from: "remoteA",
					get: () => Promise.resolve(() => "unloaded"),
					useIn: []
				}
			},
			loaded: {
				"1.0.0": loadedShared
			},
			loading: {
				"1.0.0": loadingShared
			},
			other: {
				"1.0.0": otherRemoteShared
			}
		}
	};

	try {
		await clearAndRegisterRemoteA("v3");
		const scope = shareMap[instanceId].default;
		expect(scope.unloaded["1.0.0"]).toBeUndefined();
		expect(scope.loaded["1.0.0"]).toBe(loadedShared);
		expect(scope.loading["1.0.0"]).toBeUndefined();
		expect(scope.other["1.0.0"]).toBe(otherRemoteShared);

		loadingDeferred.resolve(() => "loading");
		await staleLoading;
		expect(scope.loading["1.0.0"]).toBeUndefined();
	} finally {
		if (previous) {
			shareMap[instanceId] = previous;
		} else {
			delete shareMap[instanceId];
		}
	}
});

it("should keep cache sizes stable across repeated clear and reload", async () => {
	const moduleCacheSizes = [];
	const federationModuleCacheSizes = [];

	for (const version of ["v2", "v3", "v4"]) {
		await clearAndRegisterRemoteA(version);
		expect(await renderBothPages()).toEqual({
			pageA: `pageA:./A:${version}`,
			pageB: `pageB:./B:${version}`
		});
		moduleCacheSizes.push(getTrackedModuleCacheSize());
		federationModuleCacheSizes.push(
			getFederationInstance().moduleCache?.size || 0
		);
	}

	expect(new Set(moduleCacheSizes).size).toBe(1);
	expect(new Set(federationModuleCacheSizes).size).toBe(1);
});
