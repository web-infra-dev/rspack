import fs from 'node:fs';
import path from 'node:path';
import type { Compiler } from '../Compiler';
import type { CacheNormalized } from './normalization';

type Logger = ReturnType<Compiler['getInfrastructureLogger']>;

const SKIP = 'Skipping portable cache seed discovery:';

function realLocation(location: string): string {
  if (fs.existsSync(location)) return fs.realpathSync.native(location);
  const parent = path.dirname(location);
  return parent === location
    ? location
    : path.join(realLocation(parent), path.basename(location));
}

function readText(file: string): string | undefined {
  try {
    return fs.readFileSync(file, 'utf8');
  } catch {
    return undefined;
  }
}

function findWorktreeTop(root: string): string | undefined {
  for (let dir = root; ; dir = path.dirname(dir)) {
    if (fs.existsSync(path.join(dir, '.git'))) return dir;
    if (path.dirname(dir) === dir) return undefined;
  }
}

function findCommonDir(top: string, logger: Logger): string | undefined {
  const git = path.join(top, '.git');
  if (fs.statSync(git).isDirectory()) return git;
  const gitfile = readText(git);
  if (gitfile === undefined) {
    logger.debug(`${SKIP} cannot read ${git}.`);
    return undefined;
  }
  const match = /^gitdir:\s*(.+?)\s*$/m.exec(gitfile);
  if (!match) {
    logger.debug(`${SKIP} ${git} has no gitdir line.`);
    return undefined;
  }
  const gitdir = path.resolve(top, match[1]);
  const commondir = readText(path.join(gitdir, 'commondir'));
  if (commondir === undefined) {
    logger.debug(
      `Portable cache seed discovery: ${gitdir} has no commondir, using it as the common directory.`,
    );
    return gitdir;
  }
  return path.resolve(gitdir, commondir.trim());
}

function isBare(common: string): boolean {
  let core = false;
  for (const line of (readText(path.join(common, 'config')) ?? '').split(
    /\r?\n/,
  )) {
    const section = /^\s*\[([^\]]+)\]/.exec(line);
    if (section) core = section[1].trim().toLowerCase() === 'core';
    else if (
      core &&
      /^\s*bare(?:\s*=\s*"?(true|yes|on|1)"?)?\s*(?:[#;].*)?$/i.test(line)
    )
      return true;
  }
  return false;
}

function worktreeTops(common: string): string[] {
  const tops: string[] = [];
  if (path.basename(common) === '.git' && !isBare(common))
    tops.push(path.dirname(common));
  const registry = path.join(common, 'worktrees');
  if (!fs.existsSync(registry)) return tops;
  for (const entry of fs.readdirSync(registry)) {
    const admin = path.join(registry, entry);
    const gitfile = readText(path.join(admin, 'gitdir'));
    if (gitfile !== undefined)
      tops.push(path.dirname(path.resolve(admin, gitfile.trim())));
  }
  return tops;
}

export function findSeedDonors(
  cache: Extract<CacheNormalized, { type: 'persistent' }>,
  compiler: Compiler,
): string[] | undefined {
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
    const realRoot = realLocation(root);
    const location = realLocation(path.resolve(cache.storage.location!));
    const fromRoot = path.relative(realRoot, location);
    if (
      fromRoot === '..' ||
      fromRoot.startsWith(`..${path.sep}`) ||
      path.isAbsolute(fromRoot)
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
      return donor === location ? undefined : [donor];
    }
    const top = findWorktreeTop(realRoot);
    if (!top) {
      logger.debug(`${SKIP} no .git at or above ${realRoot}.`);
      return;
    }
    const common = findCommonDir(top, logger);
    if (!common) return;
    const relative = path.relative(top, location);
    const donors: string[] = [];
    for (const candidate of worktreeTops(common)) {
      try {
        const candidateTop = fs.realpathSync.native(candidate);
        if (candidateTop === top) continue;
        const candidateLocation = realLocation(
          path.join(candidateTop, relative),
        );
        if (
          candidateLocation !== location &&
          fs.statSync(candidateLocation).isDirectory()
        )
          donors.push(candidateLocation);
      } catch {
        // Worktrees without a cache, or removed during discovery, are not donors.
      }
    }
    if (!donors.length) {
      logger.debug(`${SKIP} no other worktree of ${common} has ${relative}.`);
      return;
    }
    return donors;
  } catch (error) {
    logger.debug(`${SKIP} ${error instanceof Error ? error.message : error}`);
    return;
  }
}
