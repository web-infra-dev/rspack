import fs from 'node:fs';
import path from 'node:path';
import type { Compiler } from '../Compiler';
import type { CacheNormalized } from './normalization';

function realLocation(location: string): string {
  if (fs.existsSync(location)) return fs.realpathSync.native(location);
  const parent = path.dirname(location);
  return parent === location
    ? location
    : path.join(realLocation(parent), path.basename(location));
}

function worktrees(root: string): string[] {
  const git = path.join(root, '.git');
  let common = git;
  if (fs.statSync(git).isFile()) {
    const match = /^gitdir:\s*(.+)\s*$/m.exec(fs.readFileSync(git, 'utf8'));
    if (!match) return [];
    const gitdir = path.resolve(root, match[1].trim());
    common = path.resolve(
      gitdir,
      fs.readFileSync(path.join(gitdir, 'commondir'), 'utf8').trim(),
    );
  }
  common = fs.realpathSync.native(common);
  const candidates: string[] = [];
  let core = false;
  let bare = false;
  for (const line of fs
    .readFileSync(path.join(common, 'config'), 'utf8')
    .split(/\r?\n/)) {
    const section = /^\s*\[([^\]]+)\]/.exec(line);
    if (section) core = section[1].trim().toLowerCase() === 'core';
    else if (
      core &&
      /^\s*bare(?:\s*=\s*"?(true|yes|on|1)"?)?\s*(?:[#;].*)?$/i.test(line)
    )
      bare = true;
  }
  if (path.basename(common) === '.git' && !bare)
    candidates.push(path.dirname(common));
  const registry = path.join(common, 'worktrees');
  if (fs.existsSync(registry)) {
    for (const entry of fs.readdirSync(registry)) {
      const admin = path.join(registry, entry);
      try {
        if (
          fs.existsSync(path.join(admin, 'locked')) ||
          fs.existsSync(path.join(admin, 'prunable'))
        )
          continue;
        const gitfile = fs
          .readFileSync(path.join(admin, 'gitdir'), 'utf8')
          .trim();
        candidates.push(path.dirname(path.resolve(admin, gitfile)));
      } catch {
        // Worktrees may be removed while discovery is running.
      }
    }
  }
  return candidates;
}

export function findCacheSeed(
  cache: Extract<CacheNormalized, { type: 'persistent' }>,
  compiler: Compiler,
): string | undefined {
  const experiment = compiler.options.experiments.portableCache;
  if (
    !experiment ||
    experiment.seed === false ||
    compiler.options.experiments.newCache ||
    compiler.intermediateFileSystem
  )
    return;
  const root = typeof cache.portable === 'object' && cache.portable.root;
  if (!root) return;
  const logger = compiler.getInfrastructureLogger('rspack.persistentCache');
  try {
    const location = realLocation(path.resolve(cache.storage.location!));
    const relative = path.relative(root, location);
    if (
      relative === '..' ||
      relative.startsWith(`..${path.sep}`) ||
      path.isAbsolute(relative)
    ) {
      logger.warn(
        'Skipping portable cache seeding: cache storage.location must be inside the portable root so each worktree owns its cache directory.',
      );
      return;
    }
    if (experiment.seedFrom !== undefined) {
      const donor = realLocation(
        path.resolve(compiler.context, experiment.seedFrom),
      );
      return donor === location ? undefined : donor;
    }
    let newest = -Infinity;
    let donor: string | undefined;
    for (const candidate of worktrees(root)) {
      try {
        const candidateRoot = fs.realpathSync.native(candidate);
        if (candidateRoot === root || !fs.statSync(candidateRoot).isDirectory())
          continue;
        const candidateLocation = realLocation(
          path.join(candidateRoot, relative),
        );
        if (candidateLocation === location) continue;
        const modified = fs.statSync(
          path.join(candidateLocation, '_meta'),
        ).mtimeMs;
        if (modified > newest) {
          newest = modified;
          donor = candidateLocation;
        }
      } catch {
        // An unavailable donor must not prevent a cold build.
      }
    }
    return donor;
  } catch {
    return;
  }
}
