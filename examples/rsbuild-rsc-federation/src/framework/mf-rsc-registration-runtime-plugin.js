const CLIENT_REFERENCE_EXPORT = '*';
const DEFAULT_RSC_MANIFEST = {
  serverManifest: {},
  clientManifest: {},
  serverConsumerModuleMap: {},
  moduleLoading: { prefix: '' },
  entryCssFiles: {},
  entryJsFiles: [],
};

function stableHash(input) {
  let hash = 2166136261;
  for (let index = 0; index < input.length; index += 1) {
    hash ^= input.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(36);
}

function createVirtualModuleId(kind, key) {
  return `mf_rsc_${kind}_${stableHash(key)}`;
}

function toUniqueStringArray(items) {
  const deduped = new Set();
  for (const item of items) {
    if (typeof item === 'string' && item) {
      deduped.add(item);
    }
  }
  return [...deduped];
}

function normalizeExportedModule(moduleValue) {
  if (
    moduleValue &&
    (typeof moduleValue === 'object' || typeof moduleValue === 'function')
  ) {
    if ('default' in moduleValue) {
      return moduleValue;
    }
    return Object.assign({ default: moduleValue }, moduleValue);
  }
  return { default: moduleValue };
}

function normalizeStringArray(value) {
  if (!Array.isArray(value)) {
    return [];
  }
  const deduped = new Set();
  for (const item of value) {
    if (typeof item === 'string' && item) {
      deduped.add(item);
    }
  }
  return [...deduped];
}

function normalizeServerActions(value) {
  if (!Array.isArray(value)) {
    return [];
  }
  const actions = [];
  const deduped = new Set();
  for (const item of value) {
    if (
      item &&
      typeof item === 'object' &&
      typeof item.id === 'string' &&
      item.id &&
      typeof item.name === 'string' &&
      item.name
    ) {
      const key = `${item.id}:${item.name}`;
      if (!deduped.has(key)) {
        deduped.add(key);
        actions.push({ id: item.id, name: item.name });
      }
    }
  }
  return actions;
}

function createRscRegistrationPayload(manifestJson) {
  const exposes = Array.isArray(manifestJson?.exposes)
    ? manifestJson.exposes
    : [];
  const shared = Array.isArray(manifestJson?.shared) ? manifestJson.shared : [];
  const exposePayload = [];
  const sharedPayload = [];

  for (const expose of exposes) {
    const rscMeta = expose?.rsc;
    if (!rscMeta) {
      continue;
    }
    const clientReferences = normalizeStringArray(rscMeta.clientReferences);
    const serverActions = normalizeServerActions(rscMeta.serverActions);
    if (clientReferences.length === 0 && serverActions.length === 0) {
      continue;
    }
    const moduleIdentity =
      rscMeta.lookup || rscMeta.resource || expose.path || expose.name || '.';
    exposePayload.push({
      exposePath: expose.path,
      moduleIdentity,
      manifestKeys: toUniqueStringArray([
        rscMeta.resource,
        rscMeta.lookup,
        moduleIdentity,
      ]),
      clientReferences,
      serverActions,
    });
  }

  for (const sharedItem of shared) {
    const rscMeta = sharedItem?.rsc;
    if (!rscMeta || typeof sharedItem?.name !== 'string' || !sharedItem.name) {
      continue;
    }
    const clientReferences = normalizeStringArray(rscMeta.clientReferences);
    const serverActions = normalizeServerActions(rscMeta.serverActions);
    if (clientReferences.length === 0 && serverActions.length === 0) {
      continue;
    }
    const moduleIdentity =
      rscMeta.lookup || rscMeta.resource || sharedItem.name;
    sharedPayload.push({
      shareKey: sharedItem.name,
      moduleIdentity,
      manifestKeys: toUniqueStringArray([
        rscMeta.resource,
        rscMeta.lookup,
        moduleIdentity,
      ]),
      clientReferences,
      serverActions,
    });
  }

  return {
    sourceId: manifestJson?.id || manifestJson?.name || '',
    buildVersion: manifestJson?.metaData?.buildInfo?.buildVersion || '',
    exposePayload,
    sharedPayload,
  };
}

function buildManifestIdentityKeys({
  remoteAlias,
  remoteName,
  remoteEntry,
  remoteVersion,
  remoteBuildVersion,
  manifestUrl,
  manifestJson,
}) {
  const keys = [
    remoteAlias && `alias:${remoteAlias}`,
    remoteName && `name:${remoteName}`,
    remoteEntry && `entry:${remoteEntry}`,
    remoteVersion && remoteName && `version:${remoteName}:${remoteVersion}`,
    remoteBuildVersion &&
      remoteName &&
      `build:${remoteName}:${remoteBuildVersion}`,
    manifestUrl && `manifestUrl:${manifestUrl}`,
    manifestJson?.id && `manifestId:${manifestJson.id}`,
    manifestJson?.name && `manifestName:${manifestJson.name}`,
    manifestJson?.metaData?.buildInfo?.buildVersion &&
      manifestJson?.name &&
      `manifestBuild:${manifestJson.name}:${manifestJson.metaData.buildInfo.buildVersion}`,
  ];
  return toUniqueStringArray(keys);
}

function ensureRscManifest() {
  const existing = __webpack_require__.rscM;
  if (existing) {
    existing.serverManifest ||= {};
    existing.clientManifest ||= {};
    existing.serverConsumerModuleMap ||= {};
    existing.moduleLoading ||= { prefix: '' };
    existing.entryCssFiles ||= {};
    existing.entryJsFiles ||= [];
    return existing;
  }
  __webpack_require__.rscM = { ...DEFAULT_RSC_MANIFEST };
  return __webpack_require__.rscM;
}

function ensureClientManifestEntry(moduleKey, moduleId) {
  if (!moduleKey || typeof moduleKey !== 'string') {
    return;
  }
  const rscManifest = ensureRscManifest();
  rscManifest.clientManifest[moduleKey] = {
    id: moduleId,
    name: CLIENT_REFERENCE_EXPORT,
    chunks: [],
    async: true,
  };
  rscManifest.serverConsumerModuleMap[moduleId] = {
    [CLIENT_REFERENCE_EXPORT]: {
      id: moduleId,
      name: CLIENT_REFERENCE_EXPORT,
      chunks: [],
      async: true,
    },
  };
}

function ensureServerActionManifestEntry(actionId, moduleId) {
  const rscManifest = ensureRscManifest();
  rscManifest.serverManifest[actionId] = {
    id: moduleId,
    name: actionId,
    chunks: [],
    async: true,
  };
}

function ensureModuleFactory(moduleId, createFactory) {
  __webpack_require__.m[moduleId] = createFactory;
}

async function readManifestJson(origin, manifestUrl) {
  if (!manifestUrl) {
    return undefined;
  }

  const snapshotCache = origin?.snapshotHandler?.manifestCache;
  if (snapshotCache?.get) {
    const cachedManifest = snapshotCache.get(manifestUrl);
    if (cachedManifest) {
      return cachedManifest;
    }
  }

  let response = await origin?.loaderHook?.lifecycle?.fetch?.emit?.(
    manifestUrl,
    {},
  );
  const hasResponseClass = typeof Response !== 'undefined';
  if (!response || (hasResponseClass && !(response instanceof Response))) {
    if (typeof fetch !== 'function') {
      return undefined;
    }
    response = await fetch(manifestUrl, {});
  }
  if (!response || typeof response.json !== 'function') {
    return undefined;
  }

  const manifestJson = await response.json();
  if (snapshotCache?.set) {
    snapshotCache.set(manifestUrl, manifestJson);
  }
  return manifestJson;
}

function pickManifestFromSnapshotCache(origin, remoteName) {
  const snapshotCache = origin?.snapshotHandler?.manifestCache;
  if (!snapshotCache?.values || !remoteName) {
    return undefined;
  }
  for (const cachedManifest of snapshotCache.values()) {
    if (!cachedManifest || typeof cachedManifest !== 'object') {
      continue;
    }
    if (
      cachedManifest.name === remoteName ||
      cachedManifest.id === remoteName
    ) {
      return cachedManifest;
    }
  }
  return undefined;
}

function toExposeRequest(remoteAlias, exposePath) {
  const exposeKey =
    typeof exposePath === 'string' ? exposePath.replace(/^\.\//, '') : '';
  return exposeKey ? `${remoteAlias}/${exposeKey}` : remoteAlias;
}

export default function mfRscRegistrationRuntimePlugin() {
  const registrationStateByAlias = new Map();
  const stagedPayloadByIdentity = new Map();

  const loadRemoteModule = (request) =>
    __webpack_require__.federation.instance
      .loadRemote(request)
      .then((remoteModule) => normalizeExportedModule(remoteModule));

  const loadSharedModule = (shareKey) =>
    __webpack_require__.federation.instance
      .loadShare(shareKey)
      .then((sharedFactory) => {
        if (!sharedFactory) {
          throw new Error(
            `[mf-rsc-registration-runtime-plugin] Unable to resolve shared module "${shareKey}".`,
          );
        }
        return normalizeExportedModule(sharedFactory());
      });

  const registerClientReferenceModule = (moduleId, exportNames, loadModule) => {
    ensureModuleFactory(moduleId, function federatedRscClientReference(module) {
      module.exports = loadModule().then((loadedModule) => {
        if (exportNames.length === 0 || exportNames.includes('*')) {
          return loadedModule;
        }
        const selectedExports = {};
        for (const exportName of exportNames) {
          if (exportName === 'default') {
            selectedExports.default = loadedModule.default;
          } else if (exportName === '*') {
            Object.assign(selectedExports, loadedModule);
          } else {
            selectedExports[exportName] = loadedModule[exportName];
          }
        }
        if (
          exportNames.includes('default') &&
          !Object.prototype.hasOwnProperty.call(selectedExports, 'default')
        ) {
          selectedExports.default = loadedModule.default;
        }
        return selectedExports;
      });
    });
  };

  const registerServerActionModule = (moduleId, serverActions, loadModule) => {
    ensureModuleFactory(moduleId, function federatedRscServerActions(module) {
      const actionExports = {};
      for (const action of serverActions) {
        actionExports[action.id] = async (...args) => {
          const loadedModule = await loadModule();
          const actionHandler = loadedModule[action.name];
          if (typeof actionHandler !== 'function') {
            throw new Error(
              `[mf-rsc-registration-runtime-plugin] Action "${action.name}" not found in loaded module.`,
            );
          }
          return actionHandler(...args);
        };
      }
      module.exports = actionExports;
    });
  };

  const cacheStagedPayload = (identityKeys, payload) => {
    for (const key of identityKeys) {
      stagedPayloadByIdentity.set(key, payload);
    }
  };

  const getStagedPayload = (identityKeys) => {
    for (const key of identityKeys) {
      const stagedPayload = stagedPayloadByIdentity.get(key);
      if (stagedPayload) {
        return stagedPayload;
      }
    }
    return undefined;
  };

  const clearRegisteredState = (registeredState) => {
    if (!registeredState) {
      return;
    }

    const rscManifest = ensureRscManifest();
    for (const moduleKey of registeredState.clientManifestKeys) {
      delete rscManifest.clientManifest[moduleKey];
    }
    for (const actionId of registeredState.serverActionIds) {
      delete rscManifest.serverManifest[actionId];
    }
    for (const moduleId of registeredState.moduleIds) {
      delete rscManifest.serverConsumerModuleMap[moduleId];
      delete __webpack_require__.m[moduleId];
      if (__webpack_require__.c) {
        delete __webpack_require__.c[moduleId];
      }
    }
  };

  const createRegistrationFingerprint = (payload) =>
    JSON.stringify({
      id: payload.sourceId,
      buildVersion: payload.buildVersion,
      exposePayload: payload.exposePayload.map((item) => ({
        exposePath: item.exposePath,
        moduleIdentity: item.moduleIdentity,
        manifestKeys: item.manifestKeys,
        clientReferences: item.clientReferences,
        serverActions: item.serverActions,
      })),
      sharedPayload: payload.sharedPayload.map((item) => ({
        pkgName: item.pkgName,
        shareKey: item.shareKey,
        moduleIdentity: item.moduleIdentity,
        manifestKeys: item.manifestKeys,
        clientReferences: item.clientReferences,
        serverActions: item.serverActions,
      })),
    });

  const registerRscPayload = (remoteAlias, payload) => {
    const fingerprint = createRegistrationFingerprint(payload);
    const previousState = registrationStateByAlias.get(remoteAlias);
    if (previousState?.fingerprint === fingerprint) {
      return;
    }
    clearRegisteredState(previousState);

    const nextState = {
      fingerprint,
      clientManifestKeys: new Set(),
      serverActionIds: new Set(),
      moduleIds: new Set(),
    };

    for (const expose of payload.exposePayload) {
      const exposeRequest = toExposeRequest(remoteAlias, expose.exposePath);
      if (expose.clientReferences.length > 0) {
        const moduleId = createVirtualModuleId(
          'client_expose',
          `${remoteAlias}:${expose.moduleIdentity}`,
        );
        registerClientReferenceModule(moduleId, expose.clientReferences, () =>
          loadRemoteModule(exposeRequest),
        );
        nextState.moduleIds.add(moduleId);
        for (const moduleKey of expose.manifestKeys) {
          ensureClientManifestEntry(moduleKey, moduleId);
          nextState.clientManifestKeys.add(moduleKey);
        }
      }

      if (expose.serverActions.length > 0) {
        const moduleId = createVirtualModuleId(
          'action_expose',
          `${remoteAlias}:${expose.moduleIdentity}`,
        );
        registerServerActionModule(moduleId, expose.serverActions, () =>
          loadRemoteModule(exposeRequest),
        );
        nextState.moduleIds.add(moduleId);
        for (const action of expose.serverActions) {
          ensureServerActionManifestEntry(action.id, moduleId);
          nextState.serverActionIds.add(action.id);
        }
      }
    }

    for (const sharedItem of payload.sharedPayload) {
      if (sharedItem.clientReferences.length > 0) {
        const moduleId = createVirtualModuleId(
          'client_shared',
          `${remoteAlias}:${sharedItem.moduleIdentity}`,
        );
        registerClientReferenceModule(
          moduleId,
          sharedItem.clientReferences,
          () => loadSharedModule(sharedItem.shareKey),
        );
        nextState.moduleIds.add(moduleId);
        for (const moduleKey of sharedItem.manifestKeys) {
          ensureClientManifestEntry(moduleKey, moduleId);
          nextState.clientManifestKeys.add(moduleKey);
        }
      }

      if (sharedItem.serverActions.length > 0) {
        const moduleId = createVirtualModuleId(
          'action_shared',
          `${remoteAlias}:${sharedItem.moduleIdentity}`,
        );
        registerServerActionModule(moduleId, sharedItem.serverActions, () =>
          loadSharedModule(sharedItem.shareKey),
        );
        nextState.moduleIds.add(moduleId);
        for (const action of sharedItem.serverActions) {
          ensureServerActionManifestEntry(action.id, moduleId);
          nextState.serverActionIds.add(action.id);
        }
      }
    }

    registrationStateByAlias.set(remoteAlias, {
      fingerprint: nextState.fingerprint,
      clientManifestKeys: [...nextState.clientManifestKeys],
      serverActionIds: [...nextState.serverActionIds],
      moduleIds: [...nextState.moduleIds],
    });
  };

  const tryRegisterManifest = async (args) => {
    const remoteAlias =
      args?.pkgNameOrAlias ||
      args?.remote?.alias ||
      args?.remoteInfo?.alias ||
      args?.remote?.name;
    const remoteName = args?.remoteInfo?.name || args?.remote?.name;
    if (!remoteAlias) {
      return;
    }

    const configuredEntry = args?.remote?.entry;
    const manifestUrl =
      typeof configuredEntry === 'string' && configuredEntry.endsWith('.json')
        ? configuredEntry
        : undefined;
    const identityKeys = buildManifestIdentityKeys({
      remoteAlias,
      remoteName,
      remoteEntry: configuredEntry,
      remoteVersion: args?.remoteSnapshot?.version || args?.remoteInfo?.version,
      remoteBuildVersion:
        args?.remoteSnapshot?.buildVersion || args?.remoteInfo?.buildVersion,
      manifestUrl,
    });

    let payload = getStagedPayload(identityKeys);
    if (!payload) {
      const manifestJson =
        (manifestUrl && (await readManifestJson(args.origin, manifestUrl))) ||
        pickManifestFromSnapshotCache(args.origin, args?.remoteInfo?.name);
      if (!manifestJson) {
        return;
      }
      payload = createRscRegistrationPayload(manifestJson);
      const manifestIdentityKeys = buildManifestIdentityKeys({
        remoteAlias,
        remoteName,
        remoteEntry: configuredEntry,
        remoteVersion:
          args?.remoteSnapshot?.version || args?.remoteInfo?.version,
        remoteBuildVersion:
          args?.remoteSnapshot?.buildVersion || args?.remoteInfo?.buildVersion,
        manifestUrl,
        manifestJson,
      });
      cacheStagedPayload(manifestIdentityKeys, payload);
    }

    if (!payload) {
      return;
    }

    registerRscPayload(remoteAlias, payload);
  };

  return {
    name: 'mf-rsc-registration-runtime-plugin',
    async loadRemoteSnapshot(args) {
      if (args?.from !== 'manifest' || !args?.manifestJson) {
        return args;
      }
      const remoteAlias = args?.moduleInfo?.alias || args?.moduleInfo?.name;
      const remoteName = args?.moduleInfo?.name;
      const payload = createRscRegistrationPayload(args.manifestJson);
      const identityKeys = buildManifestIdentityKeys({
        remoteAlias,
        remoteName,
        remoteEntry: args?.moduleInfo?.entry,
        remoteVersion:
          args?.remoteSnapshot?.version || args?.moduleInfo?.version,
        remoteBuildVersion:
          args?.remoteSnapshot?.buildVersion ||
          args?.manifestJson?.metaData?.buildInfo?.buildVersion,
        manifestUrl: args?.manifestUrl,
        manifestJson: args?.manifestJson,
      });
      cacheStagedPayload(identityKeys, payload);
      return args;
    },
    async afterResolve(args) {
      await tryRegisterManifest(args);
      return args;
    },
    async onLoad(args) {
      await tryRegisterManifest(args);
      return args;
    },
  };
}
