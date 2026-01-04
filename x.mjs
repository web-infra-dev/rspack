#!/usr/bin/env zx

import 'zx/globals';

import { Command } from 'commander';

import {
  launchJestWithArgs,
  launchRspackCli,
} from './scripts/debug/launch.mjs';
import {
  createTagName,
  getCargoVersion,
} from './scripts/release/cargo-version.mjs';
import { publish_handler } from './scripts/release/publish.mjs';
import { version_handler } from './scripts/release/version.mjs';

process.env.CARGO_TERM_COLOR = 'always'; // Assume every terminal that using zx supports color
process.env.FORCE_COLOR = 3; // Fix zx losing color output in subprocesses

const program = new Command();

program
  .name('Rspack Development CLI')
  .description('CLI for development of Rspack')
  .showHelpAfterError(true)
  .showSuggestionAfterError(true);

// x ready
program
  .command('ready')
  .alias('r')
  .description('ready to create a pull request, build and run all tests')
  .action(async () => {
    await $`cargo check`;
    await $`cargo lint`;
    await $`cargo test`;
    await $`pnpm install`;
    await $`pnpm run build:cli:release`;
    await $`pnpm run test:unit`;
    console.log(chalk.green('All passed.'));
  });

// x install
program
  .command('install')
  .alias('i')
  .description('install all dependencies')
  .action(async () => {
    await $`pnpm install`;
  });

// x clean
const cleanCommand = program
  .command('clean')
  .description('clean target/ directory');

// x clean all
cleanCommand.command('all').action(async () => {
  await $`./x clean rust`;
});

// x clean rust
cleanCommand
  .command('rust')
  .description('clean target/ directory')
  .action(async () => {
    await $`cargo clean`;
  });

// x build
const buildCommand = program.command('build').alias('b').description('build');
const watchCommand = program.command('watch').alias('w').description('watch');

buildCommand
  .option('-a', 'build all')
  .option('-b', 'build rust binding')
  .option('-j', 'build js packages')
  .option('-r', 'release')
  .option('-f', 'force')
  .action(async ({ a, b = a, j = a, r, f }) => {
    const mode = r ? 'release' : 'dev';
    try {
      if (b === undefined && j === undefined) {
        b = j = true;
      }
      b && (await $`pnpm --filter @rspack/binding build:${mode}`);
      j && (await $`pnpm --filter "@rspack/*" build ${f ? '--force' : ''}`);
    } catch (e) {
      process.exit(e.exitCode);
    }
  });

watchCommand
  .option('-a', 'watch all')
  .option('-b', 'watch rust binding')
  .option('-j', 'watch js packages')
  .option('-r', 'release')
  .action(async ({ a, b = a, j = a, r }) => {
    const mode = r ? 'release' : 'dev';
    try {
      b && (await $`pnpm --filter @rspack/binding watch:${mode}`);
      j && (await $`pnpm --filter "@rspack/*" watch`);
    } catch (e) {
      process.exit(e.exitCode);
    }
  });

// x build binding
buildCommand
  .command('binding')
  .description('build rust binding')
  .action(async () => {
    await $`pnpm --filter @rspack/binding build:dev`;
  });

// x build js
buildCommand
  .command('js')
  .description('build js packages')
  .action(async () => {
    await $`pnpm --filter "@rspack/*" build`;
  });

// x test
const testCommand = program.command('test').alias('t').description('test');

// x test rust
testCommand
  .command('rust')
  .description('run cargo tests')
  .action(async () => {
    await $`cargo test`;
  });

// x test unit
testCommand
  .command('unit')
  .description('run all unit tests')
  .action(async () => {
    await $`./x build js`;
    await $`pnpm --filter "@rspack/*" test`;
  });

// x test ci
testCommand
  .command('ci')
  .description('run tests for ci')
  .action(async () => {
    await $`./x test unit`;
  });

// x api-extractor
const extractorCommand = program
  .command('api-extractor')
  .alias('ae')
  .description('api extractor');

extractorCommand
  .command('update')
  .description('update api extractor snapshots')
  .action(async () => {
    await $`pnpm -w build:js`;
    await $`pnpm --filter "@rspack/*" api-extractor --local`;
  });

extractorCommand
  .command('ci')
  .description('test api extractor snapshots')
  .action(async () => {
    try {
      await $`pnpm --filter "@rspack/*" api-extractor:ci`;
    } catch (e) {
      console.error(
        `Api-extractor testing failed. Did you forget to update the snapshots locally?
Run the command below locally to fix this error (in the *ROOT* of rspack workspace).
$ ./x api-extractor update`,
      );
      process.exit(e.exitCode);
    }
  });

// x rspack / x rs
const rspackCommand = program.command('rspack').alias('rs').description(`
  $ x rspack -- [your-rspack-cli-args...]
  $ x rspack --debug -- build
  $ x rs -d -- build
  $ x rsd -- build
`);

rspackCommand
  .option('-d, --debug', 'Launch debugger in VSCode')
  .argument('[args...]', 'Arguments pass through to rspack cli')
  .action(async ({ debug }) => {
    try {
      if (!debug) {
        await $`pnpm exec rspack ${getVariadicArgs()}`;
        return;
      }
      await launchRspackCli(getVariadicArgs());
    } catch (e) {
      process.exit(e.exitCode);
    }
  });

// x rsd
program
  .command('rspack-debug')
  .alias('rsd')
  .description('Alias for `x rspack --debug`')
  .argument('[args...]', 'Arguments pass through to rspack cli')
  .action(async () => {
    await launchRspackCli(getVariadicArgs());
  });

// x jest / x j
const jestCommand = program.command('jest').alias('j').description(`
  $ x jest -- [your-jest-args...]
  $ x jest --debug -- -t <test-name-pattern>
  $ x j -d -- [test-path-pattern]
  $ x jd -- [your-jest-args...]
`);

jestCommand
  .option('-d, --debug', 'Launch debugger in VSCode')
  .argument('[args...]', 'Arguments pass through to rspack cli')
  .action(async ({ debug }) => {
    if (!debug) {
      await $`pnpm exec jest ${getVariadicArgs()}`;
      return;
    }
    await launchJestWithArgs(getVariadicArgs());
  });

// x jd
program
  .command('jest-debug')
  .alias('jd')
  .description('Alias for `x jest --debug`')
  .argument('[args...]', 'Arguments pass through to rspack cli')
  .action(async () => {
    await launchJestWithArgs(getVariadicArgs());
  });

program
  .command('crate-version')
  .argument(
    '<bump>',
    'bump type (major|minor|patch|premajor|preminor|prepatch|skip|prerelease|custom)',
  )
  .argument('[custom]', "custom version when bump is 'custom'")
  .description('update crate version with cargo-workspaces')
  .action(async (bump, custom) => {
    await $`which cargo-workspaces || echo "cargo-workspaces is not installed, please install it first with \`cargo install cargo-workspaces\`"`; // Validate bump type
    const validBumps = [
      'major',
      'minor',
      'patch',
      'premajor',
      'preminor',
      'prepatch',
      'skip',
      'prerelease',
      'custom',
    ];
    if (!validBumps.includes(bump)) {
      throw new Error(
        `Invalid bump type: ${bump}. Valid types: ${validBumps.join(', ')}`,
      );
    }

    // Validate custom version when bump is 'custom'
    if (bump === 'custom' && !custom) {
      throw new Error("Custom version is required when bump type is 'custom'");
    }

    const args = [
      'workspaces',
      'version',
      '--exact', // Use exact version `=`
      '--allow-branch',
      '{release*,chore/release*}', // Specify which branches to allow from
      '--no-git-push', // Do not push generated commit and tags to git remote
      '--no-git-tag', // Do not tag versions in git, we will tag them in `crate-publish`
      '--force',
      'rspack*', // Always include targeted crates matched by glob even when there are no changes
      '--yes', // Skip confirmation prompt
      bump, // Bump type
    ];

    // Add custom version if provided
    if (bump === 'custom' && custom) {
      args.push(custom);
    }

    await $`cargo ${args}`;
  });

program
  .command('crate-publish')
  .description('publish crate with cargo-workspaces')
  .option('--token <token>', 'The token to use for accessing the registry')
  .option('--dry-run', 'Runs in dry-run mode')
  .option('--push-tags', 'Push tags to git repository')
  .action(async (options) => {
    await $`which cargo-workspaces || echo "cargo-workspaces is not installed, please install it first with \`cargo install cargo-workspaces\`"`;

    const args = [
      'workspaces',
      'publish',
      '--publish-as-is', // Publish crates from the current commit without versioning
      '--publish-interval', // Number of seconds to wait between publish attempts
      '5', // 5 seconds to wait between publish attempts to make sure the previous crate is published
      '--no-remove-dev-deps', // Do not remove dev-dependencies from `Cargo.lock`, otherwise the `Cargo.toml` will be updated and dirty check will fail
      '--no-verify', // Skip verification of the workspace. This was pre-checked by `release-crates.yml` with `cargo check` with `separated` strategy
      // Commented `--locked` flag
      // because some dev-deps refer to workspace members with versions rspack_swc_plugin_ts_collector
      // and these workspace members are to be removed in release. This would cause `Cargo.lock` to be updated.
      // "--locked" // Assert that `Cargo.lock` will remain unchanged
    ];

    if (options.token) {
      args.push('--token', options.token); // The token to use for accessing the registry
    }

    if (options.dryRun) {
      args.push('--dry-run'); // Runs in dry-run mode
    }

    await $`cargo ${args}`;

    // Push tags functionality
    if (options.pushTags) {
      try {
        const version = getCargoVersion();
        const tagName = createTagName(version);

        console.info('Configuring git user for tag push...');
        await $`git config --global --add safe.directory /github/workspace`;
        await $`git config --global user.name "github-actions[bot]"`;
        await $`git config --global user.email "github-actions[bot]@users.noreply.github.com"`;

        await $`git status`;
        console.info(`Creating and pushing tag: ${tagName}`);
        await $`git tag ${tagName} -m ${tagName}`;
        await $`git push origin --follow-tags`;

        console.info(`Successfully pushed tag: ${tagName}`);
      } catch (error) {
        console.error('Error during tag push:', error);
        throw error;
      }
    }
  });

program
  .command('version')
  .argument('<bump_version>', 'bump version to (major|minor|patch|snapshot)')
  .option('--pre <string>', 'pre-release tag')
  .description('bump version for npm and rust crates')
  .action(version_handler);

program
  .command('publish')
  .argument('<mode>', 'publish mode (snapshot|stable)')
  .requiredOption('--tag <char>', 'publish tag')
  .option(
    '--dry-run',
    'Does everything a publish would do except actually publishing to the registry',
  )
  .option('--no-dry-run', 'negative dry-run')
  .option('--push-tags', 'push tags to github')
  .option('--no-push-tags', "don't push tags to github")
  .description('publish package after version bump')
  .action(publish_handler);
let argv = process.argv.slice(2); // remove the `node` and script call
if (argv[0] && /x.mjs/.test(argv[0])) {
  // Called from `zx x.mjs`
  argv = argv.slice(1);
}
program.parse(argv, { from: 'user' });

// Get args after `--`
function getVariadicArgs() {
  const idx = argv.findIndex((c) => c === '--');
  return idx === -1 ? [] : argv.slice(idx + 1);
}
