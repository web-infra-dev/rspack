const runtime = require('../../packages/rspack/src/runtime/moduleFederationDefaultRuntime.js').default;
const {
	bundlerRuntime,
} = require('@module-federation/runtime-tools/webpack-bundler-runtime');

const magicNames = [
	'__module_federation_bundler_runtime__',
	'__module_federation_runtime_plugins__',
	'__module_federation_remote_infos__',
	'__module_federation_container_name__',
	'__module_federation_share_strategy__',
	'__module_federation_share_fallbacks__',
	'__module_federation_share_fallback_variants__',
	'__module_federation_library_type__',
	'__module_federation_runtime_require__',
];

function createRuntime({
	external,
	externalType = 'commonjs-module',
	remoteShareScope = ['primary', 'secondary'],
	containerShareScope = 'container-custom',
	hasContainer = true,
	sharedFallback,
	sharedFallbackVariants,
	consumeData,
	additionalInitScopes = [],
	scopeToSharingDataMapping = {},
} = {}) {
	const shareScopeMap = {
		primary: { tag: 'primary' },
		secondary: { tag: 'secondary' },
		'host-custom': { tag: 'host-custom' },
	};
	const runtimeRequire = () => external;
	Object.assign(runtimeRequire, {
		S: shareScopeMap,
		f: {},
		I() {},
		federation: {},
		initializeSharingData: {
			scopeToSharingDataMapping,
			additionalInitScopes,
		},
		remotesLoadingData: {
			chunkMapping: {},
			moduleIdToRemoteDataMapping: {
				remote: {
					shareScope: remoteShareScope,
					name: './module',
					externalModuleId: 'external',
					remoteName: 'remote',
				},
			},
		},
	});
	if (consumeData) {
		runtimeRequire.consumesLoadingData = {
			chunkMapping: {},
			moduleIdToConsumeDataMapping: { consume: consumeData },
		};
	}
	if (hasContainer) {
		runtimeRequire.initContainer = () => {};
		runtimeRequire.initializeExposesData = {
			moduleMap: {},
			shareScope: containerShareScope,
		};
	}

	const matchedScopes = [];
	const initializedScopes = [];
	const initContainerCalls = [];
	const remote = { name: 'remote', shareScope: remoteShareScope };
	const instance = {
		name: 'host',
		options: {
			remotes: [remote],
			shareStrategy: 'version-first',
		},
		shareScopeMap,
		sharedHandler: {
			initializeSharing(shareScope) {
				initializedScopes.push(shareScope);
				if (instance.options.remotes.some(item => item.shareScope === shareScope)) {
					matchedScopes.push(shareScope);
				}
				return [];
			},
		},
		initializeSharing(shareScope, options) {
			return this.sharedHandler.initializeSharing(shareScope, options);
		},
	};
	const localBundlerRuntime = {
		...bundlerRuntime,
		init: () => instance,
		getSharedFallbackGetter: ({ shareKey }) => shareKey,
		initContainerEntry: options => {
			initContainerCalls.push(options);
		},
	};
	const importedBundlerRuntime = {
		...localBundlerRuntime,
		bundlerRuntime: localBundlerRuntime,
	};
	const remoteInfos = {
		remote: [
			{
				alias: 'remote',
				name: externalType === 'script' ? 'remote' : undefined,
				externalType,
				shareScope: remoteShareScope,
			},
		],
	};
	const instantiate = new Function(
		...magicNames,
		`return (${runtime.toString()});`,
	);
	instantiate(
		importedBundlerRuntime,
		[],
		remoteInfos,
		'host',
		'version-first',
		sharedFallback,
		sharedFallbackVariants,
		'commonjs-module',
		runtimeRequire,
	)();

	return {
		initContainerCalls,
		initializedScopes,
		instance,
		matchedScopes,
		remote,
		runtimeRequire,
		shareScopeMap,
	};
}

describe('module federation default runtime share scopes', () => {
	it('preserves scalar custom-scope container initialization', () => {
		const { initContainerCalls, instance, runtimeRequire, shareScopeMap } =
			createRuntime({ remoteShareScope: 'host-custom' });
		const remoteEntryInitOptions = { shareScopeKeys: 'host-custom' };
		Object.defineProperty(remoteEntryInitOptions, 'shareScopeMap', {
			value: shareScopeMap,
		});

		runtimeRequire.initContainer(
			shareScopeMap['host-custom'],
			[],
			remoteEntryInitOptions,
		);

		expect(initContainerCalls).toHaveLength(1);
		expect(initContainerCalls[0]).toMatchObject({
			shareScope: shareScopeMap['host-custom'],
			shareScopeKey: 'container-custom',
			remoteEntryInitOptions,
		});
		expect(instance.options.remotes[0].shareScope).toBe('host-custom');
		expect(runtimeRequire.initContainer).toHaveLength(3);
	});

	it('expands container init only by declared additional scopes', async () => {
		const { initContainerCalls, runtimeRequire, shareScopeMap } = createRuntime({
			remoteShareScope: 'host-custom',
			additionalInitScopes: ['secondary'],
			scopeToSharingDataMapping: {
				'host-custom': [],
				secondary: [],
			},
		});
		const remoteEntryInitOptions = { shareScopeKeys: 'host-custom' };
		Object.defineProperty(remoteEntryInitOptions, 'shareScopeMap', {
			value: shareScopeMap,
		});

		await runtimeRequire.initContainer(
			shareScopeMap['host-custom'],
			[],
			remoteEntryInitOptions,
		);

		expect(initContainerCalls).toHaveLength(1);
		expect(
			initContainerCalls[0].remoteEntryInitOptions.shareScopeKeys,
		).toEqual(['host-custom', 'secondary']);
		expect(remoteEntryInitOptions.shareScopeKeys).toBe('host-custom');
		expect(
			Object.getOwnPropertyDescriptor(
				initContainerCalls[0].remoteEntryInitOptions,
				'shareScopeMap',
			).enumerable,
		).toBe(false);
	});

	it('does not post-initialize scopes owned by the container', async () => {
		const { initializedScopes, runtimeRequire, shareScopeMap } = createRuntime({
			containerShareScope: 'secondary',
			additionalInitScopes: ['secondary'],
		});
		const remoteEntryInitOptions = {
			shareScopeKeys: 'primary',
			shareScopeMap,
		};

		await runtimeRequire.initContainer(
			shareScopeMap.primary,
			[],
			remoteEntryInitOptions,
		);

		expect(initializedScopes).toEqual([]);
	});

	it.each(['commonjs-module', 'module'])(
		'initializes a legacy %s remote once with the primary array scope',
		async externalType => {
			const calls = [];
			const legacyContainer = {
				init(shareScope, initScope) {
					calls.push([shareScope, initScope]);
				},
			};
			const external =
				externalType === 'module'
					? Promise.resolve(legacyContainer)
					: legacyContainer;
			const { runtimeRequire, shareScopeMap } = createRuntime({
				external,
				externalType,
			});

			await runtimeRequire.I(['primary', 'secondary'], []);

			expect(calls).toHaveLength(1);
			expect(calls[0]).toHaveLength(2);
			expect(calls[0][0]).toBe(shareScopeMap.primary);
		},
	);

	it('matches every configured script-remote scope during version-first initialization', async () => {
		const { matchedScopes, remote, runtimeRequire } = createRuntime({
			externalType: 'script',
			hasContainer: false,
		});

		await runtimeRequire.I(['primary', 'secondary'], []);

		expect(matchedScopes).toEqual(['primary', 'secondary']);
		expect(remote.shareScope).toEqual(['primary', 'secondary']);
	});

	it('passes the full ordered scope capability to an enhanced remote once', async () => {
		const calls = [];
		const enhancedContainer = {
			init(...args) {
				calls.push(args);
			},
		};
		const { runtimeRequire, shareScopeMap } = createRuntime({
			external: enhancedContainer,
		});

		await runtimeRequire.I(['primary', 'secondary'], []);

		expect(calls).toHaveLength(1);
		expect(calls[0]).toHaveLength(3);
		expect(calls[0][0]).toBe(shareScopeMap.primary);
		expect(calls[0][2].shareScopeKeys).toEqual(['primary', 'secondary']);
	});

	it('initializes a frozen legacy container without proxying its exports', async () => {
		const calls = [];
		const external = Object.freeze({
			init(...args) {
				calls.push(args);
			},
		});
		const { runtimeRequire, shareScopeMap } = createRuntime({ external });

		await runtimeRequire.I(['primary', 'secondary'], []);

		expect(calls).toHaveLength(1);
		expect(calls[0][0]).toBe(shareScopeMap.primary);
	});

	it('does not retry a legacy container whose init throws synchronously', async () => {
		let calls = 0;
		const external = {
			init() {
				calls += 1;
				throw new Error('legacy init failed');
			},
		};
		const { runtimeRequire } = createRuntime({ external });

		await runtimeRequire.I(['primary', 'secondary'], []);

		expect(calls).toBe(1);
	});

	it('selects tree-shaking fallbacks by scope and layer identity', () => {
		const sharedFallback = {
			react: [
				['legacy.js', '1.0.0', 'legacy'],
				['server.js', '1.0.0', 'server'],
			],
		};
		const { runtimeRequire } = createRuntime({
			sharedFallback,
			sharedFallbackVariants: {
				react: [
					{
						entry: 'legacy.js',
						version: '1.0.0',
						globalName: 'legacy',
						shareScope: 'primary',
						import: 'react',
					},
					{
						entry: 'server.js',
						version: '1.0.0',
						globalName: 'server',
						shareScope: 'primary',
						layer: 'server',
						import: 'react-server',
					},
				],
			},
			consumeData: {
				shareKey: 'react',
				shareScope: 'primary',
				layer: 'server',
				import: 'react-server',
			},
		});

		const fallbackKey =
			runtimeRequire.federation.consumesLoadingModuleToHandlerMapping.consume
				.getter;
		expect(fallbackKey).toBe('react\0consume');
		expect(runtimeRequire.federation.sharedFallback[fallbackKey]).toEqual([
			['server.js', '1.0.0', 'server'],
		]);
		expect(sharedFallback.react).toHaveLength(2);
	});

	it('uses the consume fallback when no tree-shaking variant matches', () => {
		const fallback = () => 'local';
		const { runtimeRequire } = createRuntime({
			sharedFallback: { react: [['client.js', '1.0.0', 'client']] },
			sharedFallbackVariants: {
				react: [
					{
						entry: 'client.js',
						version: '1.0.0',
						globalName: 'client',
						shareScope: 'client',
						import: 'react-client',
					},
				],
			},
			consumeData: {
				shareKey: 'react',
				shareScope: 'server',
				import: 'react-server',
				fallback,
			},
		});

		expect(
			runtimeRequire.federation.consumesLoadingModuleToHandlerMapping.consume
				.getter,
		).toBe(fallback);
	});
});
