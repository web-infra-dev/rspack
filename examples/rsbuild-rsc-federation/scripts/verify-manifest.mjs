import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '..',
);
const distDir = path.join(rootDir, 'dist');

function invariant(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function findFilesByName(dir, targetName, result = []) {
  if (!fs.existsSync(dir)) {
    return result;
  }
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      findFilesByName(fullPath, targetName, result);
      continue;
    }
    if (entry.isFile() && entry.name === targetName) {
      result.push(fullPath);
    }
  }
  return result;
}

function pickBestMatch(candidates) {
  if (candidates.length === 0) {
    return undefined;
  }
  const serverCandidate = candidates.find((candidate) =>
    candidate.includes(`${path.sep}server${path.sep}`),
  );
  return serverCandidate || candidates[0];
}

function readJSON(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf-8'));
}

const statsCandidates = findFilesByName(distDir, 'mf-stats.json');
const manifestCandidates = findFilesByName(distDir, 'mf-manifest.json');

invariant(
  statsCandidates.length > 0,
  `No mf-stats.json found under ${distDir}`,
);
invariant(
  manifestCandidates.length > 0,
  `No mf-manifest.json found under ${distDir}`,
);

const statsPath = pickBestMatch(statsCandidates);
const manifestPath =
  manifestCandidates.find(
    (candidate) => path.dirname(candidate) === path.dirname(statsPath),
  ) || pickBestMatch(manifestCandidates);

const stats = readJSON(statsPath);
const manifest = readJSON(manifestPath);
const clientStatsPath = path.join(distDir, 'mf-manifest.client-stats.json');
const clientManifestPath = path.join(distDir, 'mf-manifest.client.json');

invariant(
  fs.existsSync(clientStatsPath),
  `Expected client stats manifest at ${clientStatsPath}`,
);
invariant(
  fs.existsSync(clientManifestPath),
  `Expected client manifest at ${clientManifestPath}`,
);

const clientStats = readJSON(clientStatsPath);
const clientManifest = readJSON(clientManifestPath);
const containerName = 'rsbuild_host';
const sharedPackageName = 'rsbuild-rsc-federation-shared';
const sharedActionsPackageName = 'rsbuild-rsc-federation-shared/server-actions';

const sharedStats = stats.shared.find(
  (item) => item.name === sharedPackageName,
);
invariant(
  sharedStats,
  `Expected shared entry for "${sharedPackageName}" in stats`,
);
invariant(
  sharedStats.shareKey === 'rsc-shared-key',
  `Expected shared shareKey "rsc-shared-key", got "${sharedStats.shareKey}"`,
);
invariant(sharedStats.rsc, 'Expected rsc metadata on shared stats entry');
invariant(
  sharedStats.rsc.lookup === 'rsc-shared-key',
  `Expected shared lookup "rsc-shared-key", got "${sharedStats.rsc.lookup}"`,
);
invariant(
  sharedStats.rsc.clientReferences.includes('SharedClientComponent'),
  'Expected SharedClientComponent in shared clientReferences',
);
invariant(
  sharedStats.rsc.clientReferences.includes('sharedValue'),
  'Expected sharedValue in shared clientReferences',
);
invariant(
  sharedStats.rsc.clientReferences.includes('MixedClientBadge'),
  'Expected MixedClientBadge in shared clientReferences',
);
invariant(
  sharedStats.rsc.clientReferences.includes('mixedValue'),
  'Expected mixedValue in shared clientReferences',
);
invariant(
  Array.isArray(sharedStats.rsc.serverActions),
  'Expected shared serverActions to be an array',
);

const sharedActionsStats = stats.shared.find(
  (item) => item.name === sharedActionsPackageName,
);
invariant(
  sharedActionsStats,
  `Expected shared entry for "${sharedActionsPackageName}" in stats`,
);
invariant(
  sharedActionsStats.shareKey === 'rsc-shared-actions-key',
  `Expected shared actions shareKey "rsc-shared-actions-key", got "${sharedActionsStats.shareKey}"`,
);
invariant(
  sharedActionsStats.rsc,
  'Expected rsc metadata on shared actions stats entry',
);
invariant(
  sharedActionsStats.rsc.lookup === 'rsc-shared-actions-key',
  `Expected shared actions lookup "rsc-shared-actions-key", got "${sharedActionsStats.rsc.lookup}"`,
);
invariant(
  Array.isArray(sharedActionsStats.rsc.serverActions) &&
    sharedActionsStats.rsc.serverActions.length > 0,
  'Expected shared actions serverActions to be non-empty',
);

const exposeStats = stats.exposes.find((item) => item.path === './button');
invariant(exposeStats, 'Expected expose entry "./button" in stats');
invariant(exposeStats.rsc, 'Expected rsc metadata on expose stats entry');
invariant(
  exposeStats.rsc.lookup === `${containerName}/button`,
  `Expected expose lookup "${containerName}/button", got "${exposeStats.rsc.lookup}"`,
);
invariant(
  exposeStats.rsc.clientReferences.includes('default'),
  'Expected expose clientReferences to include "default"',
);
invariant(
  exposeStats.rsc.serverActions.length === 0,
  'Expected client expose "./button" to not include serverActions',
);

const consumerExposeStats = stats.exposes.find(
  (item) => item.path === './consumer',
);
invariant(consumerExposeStats, 'Expected expose entry "./consumer" in stats');
invariant(
  consumerExposeStats.rsc,
  'Expected rsc metadata on expose "./consumer"',
);
invariant(
  consumerExposeStats.rsc.lookup === `${containerName}/consumer`,
  `Expected expose lookup "${containerName}/consumer", got "${consumerExposeStats.rsc.lookup}"`,
);
invariant(
  Array.isArray(consumerExposeStats.rsc.serverActions) &&
    consumerExposeStats.rsc.serverActions.length > 0,
  'Expected expose "./consumer" serverActions to be non-empty',
);

const composedExposeStats = stats.exposes.find(
  (item) => item.path === './composed',
);
invariant(composedExposeStats, 'Expected expose entry "./composed" in stats');
invariant(
  composedExposeStats.rsc,
  'Expected rsc metadata on expose "./composed"',
);
invariant(
  composedExposeStats.rsc.lookup === `${containerName}/composed`,
  `Expected expose lookup "${containerName}/composed", got "${composedExposeStats.rsc.lookup}"`,
);
invariant(
  composedExposeStats.rsc.clientReferences.includes('default'),
  'Expected expose "./composed" clientReferences to include "default"',
);

const serverMixedExposeStats = stats.exposes.find(
  (item) => item.path === './server-mixed',
);
invariant(
  serverMixedExposeStats,
  'Expected expose entry "./server-mixed" in stats',
);
invariant(
  serverMixedExposeStats.rsc,
  'Expected rsc metadata on expose "./server-mixed"',
);
invariant(
  serverMixedExposeStats.rsc.lookup === `${containerName}/server-mixed`,
  `Expected expose lookup "${containerName}/server-mixed", got "${serverMixedExposeStats.rsc.lookup}"`,
);
invariant(
  Array.isArray(serverMixedExposeStats.rsc.serverActions) &&
    serverMixedExposeStats.rsc.serverActions.length > 0,
  'Expected expose "./server-mixed" serverActions to be non-empty',
);

const expectedRemoteModules = ['button', 'consumer', 'server-mixed'];
for (const remoteModule of expectedRemoteModules) {
  const remoteStats = stats.remotes.find(
    (item) => item.alias === 'remote' && item.moduleName === remoteModule,
  );
  invariant(
    remoteStats,
    `Expected remote/${remoteModule} consumption entry in stats`,
  );
  invariant(
    remoteStats.rsc,
    `Expected rsc metadata on remote stats entry for remote/${remoteModule}`,
  );
  invariant(
    remoteStats.rsc.lookup === `remote/${remoteModule}`,
    `Expected remote lookup "remote/${remoteModule}", got "${remoteStats.rsc.lookup}"`,
  );
}

const manifestShared = manifest.shared.find(
  (item) => item.name === sharedPackageName,
);
invariant(manifestShared?.rsc, 'Expected shared rsc metadata in manifest');
invariant(
  manifestShared.shareKey === 'rsc-shared-key',
  'Manifest shared shareKey mismatch',
);
invariant(
  manifestShared.rsc.lookup === 'rsc-shared-key',
  'Manifest shared lookup mismatch',
);

const manifestSharedActions = manifest.shared.find(
  (item) => item.name === sharedActionsPackageName,
);
invariant(
  manifestSharedActions?.rsc,
  'Expected shared actions rsc metadata in manifest',
);
invariant(
  manifestSharedActions.shareKey === 'rsc-shared-actions-key',
  'Manifest shared actions shareKey mismatch',
);
invariant(
  manifestSharedActions.rsc.lookup === 'rsc-shared-actions-key',
  'Manifest shared actions lookup mismatch',
);

const manifestExpose = manifest.exposes.find(
  (item) => item.path === './button',
);
invariant(manifestExpose?.rsc, 'Expected expose rsc metadata in manifest');
invariant(
  manifestExpose.rsc.lookup === `${containerName}/button`,
  'Manifest expose lookup mismatch',
);

const manifestConsumerExpose = manifest.exposes.find(
  (item) => item.path === './consumer',
);
invariant(
  manifestConsumerExpose?.rsc,
  'Expected expose "./consumer" rsc metadata in manifest',
);
invariant(
  manifestConsumerExpose.rsc.lookup === `${containerName}/consumer`,
  'Manifest expose "./consumer" lookup mismatch',
);

const manifestComposedExpose = manifest.exposes.find(
  (item) => item.path === './composed',
);
invariant(
  manifestComposedExpose?.rsc,
  'Expected expose "./composed" rsc metadata in manifest',
);
invariant(
  manifestComposedExpose.rsc.lookup === `${containerName}/composed`,
  'Manifest expose "./composed" lookup mismatch',
);

const manifestServerMixedExpose = manifest.exposes.find(
  (item) => item.path === './server-mixed',
);
invariant(
  manifestServerMixedExpose?.rsc,
  'Expected expose "./server-mixed" rsc metadata in manifest',
);
invariant(
  manifestServerMixedExpose.rsc.lookup === `${containerName}/server-mixed`,
  'Manifest expose "./server-mixed" lookup mismatch',
);

for (const remoteModule of expectedRemoteModules) {
  const manifestRemote = manifest.remotes.find(
    (item) => item.alias === 'remote' && item.moduleName === remoteModule,
  );
  invariant(
    manifestRemote?.rsc,
    `Expected remote rsc metadata in manifest for remote/${remoteModule}`,
  );
  invariant(
    manifestRemote.rsc.lookup === `remote/${remoteModule}`,
    `Manifest remote lookup mismatch for remote/${remoteModule}`,
  );
}

const expectedSingletonShares = [
  'react',
  'react/jsx-runtime',
  'react-dom',
  'react-dom/server',
  'react-server-dom-rspack/server.node',
];
for (const shareName of expectedSingletonShares) {
  const sharedEntry = stats.shared.find((item) => item.name === shareName);
  invariant(sharedEntry, `Expected shared singleton "${shareName}" in stats`);
  invariant(
    sharedEntry.singleton === true,
    `Expected shared singleton "${shareName}" to be true`,
  );
}

invariant(
  clientStats.name === containerName,
  `Expected client stats container name "${containerName}", got "${clientStats.name}"`,
);
invariant(
  clientManifest.name === containerName,
  `Expected client manifest container name "${containerName}", got "${clientManifest.name}"`,
);
invariant(
  clientStats.exposes.some((item) => item.path === './button'),
  'Expected client stats to include expose "./button"',
);
invariant(
  clientStats.exposes.some((item) => item.path === './composed'),
  'Expected client stats to include expose "./composed"',
);
invariant(
  !clientStats.exposes.some((item) => item.path === './consumer'),
  'Did not expect client stats to include server-only expose "./consumer"',
);
invariant(
  !clientStats.exposes.some((item) => item.path === './server-mixed'),
  'Did not expect client stats to include server-only expose "./server-mixed"',
);
invariant(
  !clientManifest.exposes.some((item) => item.path === './consumer'),
  'Did not expect client manifest to include server-only expose "./consumer"',
);
invariant(
  !clientManifest.exposes.some((item) => item.path === './server-mixed'),
  'Did not expect client manifest to include server-only expose "./server-mixed"',
);
const expectedClientSingletonShares = [
  'react',
  'react/jsx-runtime',
  'react-dom',
];
for (const shareName of expectedClientSingletonShares) {
  const sharedEntry = clientStats.shared.find(
    (item) => item.name === shareName,
  );
  invariant(
    sharedEntry,
    `Expected client shared singleton "${shareName}" in client stats`,
  );
  invariant(
    sharedEntry.singleton === true,
    `Expected client shared singleton "${shareName}" to be true`,
  );
}
invariant(
  !clientStats.shared.some(
    (item) => item.name === 'rsbuild-rsc-federation-shared/server-actions',
  ),
  'Did not expect client stats to include server-only shared actions entry',
);
invariant(
  !clientManifest.shared.some(
    (item) => item.name === 'rsbuild-rsc-federation-shared/server-actions',
  ),
  'Did not expect client manifest to include server-only shared actions entry',
);

console.log('[verify-manifest] verified manifest and stats successfully');
console.log(`[verify-manifest] stats: ${statsPath}`);
console.log(`[verify-manifest] manifest: ${manifestPath}`);
console.log(`[verify-manifest] client stats: ${clientStatsPath}`);
console.log(`[verify-manifest] client manifest: ${clientManifestPath}`);
