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
		activeFactoryVersion: undefined,
		remoteEntryLoads: [],
		providerRuntime: undefined,
		remoteGets: [],
		factoryExecutions: [],
		routeExecutions: {
			pageA: 0,
			pageB: 0
		},
		blockNextGet: false,
		blockedGet: undefined,
		blockedGetObserved: undefined,
		setVersion(version) {
			this.version = version;
		},
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
				remoteGets: this.remoteGets.length,
				factoryExecutions: this.factoryExecutions.length,
				routeExecutions: { ...this.routeExecutions }
			};
		},
		wrapRemoteEntry(remoteEntry, entry) {
			const entryVersion = this.version;
			this.remoteEntryLoads.push(`${entryVersion}:${entry}`);
			return {
				init: (...args) => remoteEntry.init(...args),
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
		}
	};
	globalThis.__mfSsrClearCacheRemoteServer = remoteServer;
	return remoteServer;
}
