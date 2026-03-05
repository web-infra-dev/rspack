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

const sharedStats = stats.shared.find((item) => item.name === 'rsc-shared-key');
invariant(sharedStats, 'Expected shared entry for "rsc-shared-key" in stats');
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
  sharedStats.rsc.clientReferences.includes('sharedAction'),
  'Expected sharedAction in shared clientReferences',
);
invariant(
  sharedStats.rsc.clientReferences.includes('sharedValue'),
  'Expected sharedValue in shared clientReferences',
);
invariant(
  Array.isArray(sharedStats.rsc.serverActions) &&
    sharedStats.rsc.serverActions.length > 0,
  'Expected shared serverActions to be non-empty',
);

const exposeStats = stats.exposes.find((item) => item.path === './button');
invariant(exposeStats, 'Expected expose entry "./button" in stats');
invariant(exposeStats.rsc, 'Expected rsc metadata on expose stats entry');
invariant(
  exposeStats.rsc.lookup === 'rsbuild_container/button',
  `Expected expose lookup "rsbuild_container/button", got "${exposeStats.rsc.lookup}"`,
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
  consumerExposeStats.rsc.lookup === 'rsbuild_container/consumer',
  `Expected expose lookup "rsbuild_container/consumer", got "${consumerExposeStats.rsc.lookup}"`,
);
invariant(
  Array.isArray(consumerExposeStats.rsc.serverActions) &&
    consumerExposeStats.rsc.serverActions.length > 0,
  'Expected expose "./consumer" serverActions to be non-empty',
);

const remoteStats = stats.remotes.find(
  (item) => item.alias === 'remote' && item.moduleName === 'Button',
);
invariant(remoteStats, 'Expected remote/Button consumption entry in stats');
invariant(remoteStats.rsc, 'Expected rsc metadata on remote stats entry');
invariant(
  remoteStats.rsc.lookup === 'remote/Button',
  `Expected remote lookup "remote/Button", got "${remoteStats.rsc.lookup}"`,
);

const manifestShared = manifest.shared.find(
  (item) => item.name === 'rsc-shared-key',
);
invariant(manifestShared?.rsc, 'Expected shared rsc metadata in manifest');
invariant(
  manifestShared.rsc.lookup === 'rsc-shared-key',
  'Manifest shared lookup mismatch',
);

const manifestExpose = manifest.exposes.find(
  (item) => item.path === './button',
);
invariant(manifestExpose?.rsc, 'Expected expose rsc metadata in manifest');
invariant(
  manifestExpose.rsc.lookup === 'rsbuild_container/button',
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
  manifestConsumerExpose.rsc.lookup === 'rsbuild_container/consumer',
  'Manifest expose "./consumer" lookup mismatch',
);

const manifestRemote = manifest.remotes.find(
  (item) => item.alias === 'remote' && item.moduleName === 'Button',
);
invariant(manifestRemote?.rsc, 'Expected remote rsc metadata in manifest');
invariant(
  manifestRemote.rsc.lookup === 'remote/Button',
  'Manifest remote lookup mismatch',
);

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

console.log('[verify-manifest] verified manifest and stats successfully');
console.log(`[verify-manifest] stats: ${statsPath}`);
console.log(`[verify-manifest] manifest: ${manifestPath}`);
