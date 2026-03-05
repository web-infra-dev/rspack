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
  rscManifest.clientManifest[moduleKey] ||= {
    id: moduleId,
    name: CLIENT_REFERENCE_EXPORT,
    chunks: [],
    async: true,
  };
  rscManifest.serverConsumerModuleMap[moduleId] ||= {
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
  rscManifest.serverManifest[actionId] ||= {
    id: moduleId,
    name: actionId,
    chunks: [],
    async: true,
  };
}

function ensureModuleFactory(moduleId, createFactory) {
  __webpack_require__.m[moduleId] ||= createFactory;
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

  let response = await origin?.loaderHook?.lifecycle?.fetch?.emit?.(manifestUrl, {});
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
    if (cachedManifest.name === remoteName || cachedManifest.id === remoteName) {
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
  const registrationFingerprintByAlias = new Map();

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

  const registerRscDataFromManifest = (remoteAlias, manifestJson) => {
    const exposes = Array.isArray(manifestJson?.exposes)
      ? manifestJson.exposes
      : [];
    const shared = Array.isArray(manifestJson?.shared) ? manifestJson.shared : [];
    const fingerprint = JSON.stringify({
      id: manifestJson?.id || manifestJson?.name || '',
      buildVersion: manifestJson?.metaData?.buildInfo?.buildVersion || '',
      exposeRscKeys: exposes
        .filter((item) => item?.rsc)
        .map((item) => item.rsc.lookup || item.rsc.resource || item.path || ''),
      sharedRscKeys: shared
        .filter((item) => item?.rsc)
        .map((item) => item.rsc.lookup || item.rsc.resource || item.name || ''),
    });
    if (registrationFingerprintByAlias.get(remoteAlias) === fingerprint) {
      return;
    }

    for (const expose of exposes) {
      const rscMeta = expose?.rsc;
      if (!rscMeta) {
        continue;
      }

      const clientReferences = normalizeStringArray(rscMeta.clientReferences);
      const serverActions = normalizeServerActions(rscMeta.serverActions);
      const exposeRequest = toExposeRequest(remoteAlias, expose.path);
      const moduleIdentity = rscMeta.lookup || rscMeta.resource || exposeRequest;
      const manifestKeys = new Set(
        [rscMeta.resource, rscMeta.lookup, moduleIdentity].filter(Boolean),
      );

      if (clientReferences.length > 0) {
        const moduleId = createVirtualModuleId(
          'client_expose',
          `${remoteAlias}:${moduleIdentity}`,
        );
        registerClientReferenceModule(moduleId, clientReferences, () =>
          loadRemoteModule(exposeRequest),
        );
        for (const moduleKey of manifestKeys) {
          ensureClientManifestEntry(moduleKey, moduleId);
        }
      }

      if (serverActions.length > 0) {
        const moduleId = createVirtualModuleId(
          'action_expose',
          `${remoteAlias}:${moduleIdentity}`,
        );
        registerServerActionModule(moduleId, serverActions, () =>
          loadRemoteModule(exposeRequest),
        );
        for (const action of serverActions) {
          ensureServerActionManifestEntry(action.id, moduleId);
        }
      }
    }

    for (const sharedItem of shared) {
      const rscMeta = sharedItem?.rsc;
      if (!rscMeta || typeof sharedItem?.name !== 'string' || !sharedItem.name) {
        continue;
      }

      const shareKey = sharedItem.name;
      const clientReferences = normalizeStringArray(rscMeta.clientReferences);
      const serverActions = normalizeServerActions(rscMeta.serverActions);
      const moduleIdentity = rscMeta.lookup || rscMeta.resource || shareKey;
      const manifestKeys = new Set(
        [rscMeta.resource, rscMeta.lookup, moduleIdentity].filter(Boolean),
      );

      if (clientReferences.length > 0) {
        const moduleId = createVirtualModuleId(
          'client_shared',
          `${remoteAlias}:${moduleIdentity}`,
        );
        registerClientReferenceModule(moduleId, clientReferences, () =>
          loadSharedModule(shareKey),
        );
        for (const moduleKey of manifestKeys) {
          ensureClientManifestEntry(moduleKey, moduleId);
        }
      }

      if (serverActions.length > 0) {
        const moduleId = createVirtualModuleId(
          'action_shared',
          `${remoteAlias}:${moduleIdentity}`,
        );
        registerServerActionModule(moduleId, serverActions, () =>
          loadSharedModule(shareKey),
        );
        for (const action of serverActions) {
          ensureServerActionManifestEntry(action.id, moduleId);
        }
      }
    }

    registrationFingerprintByAlias.set(remoteAlias, fingerprint);
  };

  const tryRegisterManifest = async (args) => {
    const remoteAlias =
      args?.pkgNameOrAlias ||
      args?.remote?.alias ||
      args?.remoteInfo?.alias ||
      args?.remote?.name;
    if (!remoteAlias) {
      return;
    }

    const configuredEntry = args?.remote?.entry;
    const manifestUrl =
      typeof configuredEntry === 'string' && configuredEntry.endsWith('.json')
        ? configuredEntry
        : undefined;
    const manifestJson =
      (manifestUrl && (await readManifestJson(args.origin, manifestUrl))) ||
      pickManifestFromSnapshotCache(args.origin, args?.remoteInfo?.name);

    if (!manifestJson) {
      return;
    }

    registerRscDataFromManifest(remoteAlias, manifestJson);
  };

  return {
    name: 'mf-rsc-registration-runtime-plugin',
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
