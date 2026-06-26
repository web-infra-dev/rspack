function createDeferred() {
	let resolve;
	let reject;
	const promise = new Promise((r, j) => {
		resolve = r;
		reject = j;
	});
	return { promise, resolve, reject };
}

export function createRemoteServer() {
	const remoteServer = {
		version: "v1",
		largeRemotePayloadSize: 50000,
		activeFactoryVersion: undefined,
		remoteEntryLoads: [],
		remoteEntryClears: [],
		providerRuntime: undefined,
		remoteGets: [],
		factoryExecutions: [],
		largeRemotePayloads: [],
		routeExecutions: {
			pageA: 0,
			pageB: 0
		},
		blockNextGet: false,
		blockedGet: undefined,
		blockedGetObserved: undefined,
		setProviderRuntime(providerRuntime) {
			this.providerRuntime = providerRuntime;
		},
		clearProviderRuntime() {
			this.providerRuntime?.clear();
		},
		blockNextRemoteGet() {
			this.blockNextGet = true;
		},
		waitForBlockedRemoteGet() {
			if (this.blockedGet) {
				return Promise.resolve();
			}
			return new Promise(resolve => {
				this.blockedGetObserved = resolve;
			});
		},
		resolveBlockedRemoteGet() {
			const blockedGet = this.blockedGet;
			this.blockedGet = undefined;
			blockedGet.resolve();
		},
		rejectBlockedRemoteGet(error) {
			const blockedGet = this.blockedGet;
			this.blockedGet = undefined;
			blockedGet.reject(error);
		},
		recordRouteExecution(route) {
			this.routeExecutions[route] += 1;
		},
		snapshot() {
			return {
				remoteEntryLoads: this.remoteEntryLoads.length,
				remoteEntryClears: this.remoteEntryClears.length,
				remoteGets: this.remoteGets.length,
				factoryExecutions: this.factoryExecutions.length,
				largeRemotePayloads: this.largeRemotePayloads.length,
				routeExecutions: { ...this.routeExecutions }
			};
		},
		getRemoteEntryVersion(entry) {
			const match = String(entry).match(
				/remote-project-(v\d+)\/remoteEntry\.js/
			);
			return match?.[1] || this.version;
		},
		wrapRemoteEntry(remoteEntry, entry) {
			const entryVersion = this.getRemoteEntryVersion(entry);
			if (typeof remoteEntry.__webpack_clear_cache__ !== "function") {
				throw new Error("remoteEntry is missing __webpack_clear_cache__");
			}
			this.remoteEntryLoads.push(`${entryVersion}:${entry}`);
			return {
				init: (...args) => remoteEntry.init(...args),
				__webpack_clear_cache__: () => {
					this.remoteEntryClears.push(`${entryVersion}:${entry}`);
					remoteEntry.__webpack_clear_cache__();
				},
				get: expose => {
					this.remoteGets.push(`${entryVersion}:${expose}`);
					const getFactory = Promise.resolve(remoteEntry.get(expose)).then(
						factory => {
							return (...args) =>
								this.runRemoteFactory(entryVersion, expose, () =>
									factory(...args)
								);
						}
					);
					if (this.blockNextGet) {
						this.blockNextGet = false;
						this.blockedGet = createDeferred();
						if (this.blockedGetObserved) {
							this.blockedGetObserved();
							this.blockedGetObserved = undefined;
						}
						return this.blockedGet.promise.then(() => getFactory);
					}
					return getFactory;
				}
			};
		},
		runRemoteFactory(entryVersion, expose, run) {
			const previousVersion = this.activeFactoryVersion;
			this.activeFactoryVersion = entryVersion;
			try {
				return run();
			} finally {
				this.factoryExecutions.push(`${entryVersion}:${expose}`);
				this.activeFactoryVersion = previousVersion;
			}
		},
		createRemoteExport(expose) {
			const version = this.activeFactoryVersion || this.version;
			return `${expose}:${version}`;
		},
		createLargeRemoteExport(expose, remoteVersion) {
			const version =
				remoteVersion || this.activeFactoryVersion || this.version;
			const payload = Array.from(
				{ length: this.largeRemotePayloadSize },
				(_, index) =>
					`${expose}:${version}:${index}:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`
			);
			this.largeRemotePayloads.push({
				expose,
				version,
				size: payload.length,
				first: payload[0],
				last: payload[payload.length - 1]
			});
			return {
				value: `${expose}:${version}`,
				payload
			};
		}
	};
	globalThis.__mfSsrClearCacheRemoteServer = remoteServer;
	return remoteServer;
}
