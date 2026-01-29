import path from 'node:path';
import type {
  Chunk,
  RspackOptions,
  Stats,
  StatsCompilation,
} from '@rspack/core';
import fs from 'fs-extra';
import { escapeSep } from '../helper';
import { normalizePlaceholder } from '../helper/expect/placeholder';
import { BasicCaseCreator } from '../test/creator';
import type { ITestContext, ITestEnv } from '../type';
import { createHotProcessor, createHotRunner } from './hot';
import type { THotStepRuntimeData } from './runner';

type TTarget = RspackOptions['target'];

type TModuleGetHandler = (file: string, options: RspackOptions) => string[];

declare let global: {
  self?: {
    [key: string]: (name: string, modules: Record<string, unknown>) => void;
  };
};

const NOOP_SET = new Set();

const escapeLocalName = (str: string) => str.split(/[-<>:"/|?*.]/).join('_');

const SELF_HANDLER = (file: string, options: RspackOptions): string[] => {
  let res: string[] = [];
  const hotUpdateGlobal = (_: string, modules: Record<string, unknown>) => {
    res = Object.keys(modules);
  };
  const hotUpdateGlobalKey = escapeLocalName(
    `${options.output?.hotUpdateGlobal || 'rspackHotUpdate'}${
      options.output?.uniqueName || ''
    }`,
  );
  global.self ??= {};
  global.self[hotUpdateGlobalKey] = hotUpdateGlobal;
  require(file);
  delete global.self[hotUpdateGlobalKey];
  return res;
};

const NODE_HANDLER = (file: string): string[] => {
  return Object.keys(require(file).modules) || [];
};

const GET_MODULE_HANDLER: Record<string, TModuleGetHandler> = {
  web: SELF_HANDLER,
  webworker: SELF_HANDLER,
  'async-node': NODE_HANDLER,
  node: NODE_HANDLER,
};

type TSupportTarget = keyof typeof GET_MODULE_HANDLER;

const creators: Map<TTarget, BasicCaseCreator> = new Map();

function createHotStepProcessor(
  name: string,
  src: string,
  temp: string,
  target: TTarget,
) {
  const processor = createHotProcessor(name, src, temp, target);
  const entries: Record<string, string[]> = {};
  const hashes: string[] = [];

  function matchStepSnapshot(
    env: ITestEnv,
    context: ITestContext,
    step: number,
    options: RspackOptions,
    stats: StatsCompilation,
    runtime?: THotStepRuntimeData,
  ) {
    const getModuleHandler =
      GET_MODULE_HANDLER[options.target as TSupportTarget];
    env.expect(typeof getModuleHandler).toBe('function');

    const lastHash = hashes[hashes.length - 1];
    const snapshotPath = context.getSource(
      `__snapshots__/${options.target}/${step}.snap.txt`,
    );
    const title = `Case ${path.basename(name)}: Step ${step}`;
    const hotUpdateFile: Array<{
      name: string;
      content: string;
      modules: string[];
      runtime: string[];
    }> = [];
    const hotUpdateManifest: Array<{ name: string; content: string }> = [];
    const changedFiles: string[] =
      step === 0
        ? []
        : processor.updatePlugin
            .getModifiedFiles()
            .map((i: string) => escapeSep(path.relative(temp, i)));
    changedFiles.sort();

    const resultHashes: Record<string, string> = {
      [lastHash || 'LAST_HASH']: 'LAST_HASH',
      [stats.hash!]: 'CURRENT_HASH',
    };

    // TODO: find a better way
    // replace [runtime] to [runtime of id] to prevent worker hash
    const runtimes: Record<string, string> = {};
    for (const [id, runtime] of Object.entries(entries)) {
      if (typeof runtime === 'string') {
        if (runtime !== id) {
          runtimes[runtime] = `[runtime of ${id}]`;
        }
      } else if (Array.isArray(runtime)) {
        for (const r of runtime) {
          if (r !== id) {
            runtimes[r] = `[runtime of ${id}]`;
          }
        }
      }
    }

    const replaceContent = (rawStr: string) => {
      let str = rawStr;
      const replaceContentConfig = context.getTestConfig().snapshotContent;
      if (replaceContentConfig) {
        str = replaceContentConfig(str);
      }
      return normalizePlaceholder(
        Object.entries(resultHashes)
          .reduce((str, [raw, replacement]) => {
            return str.split(raw).join(replacement);
          }, str)
          .replace(/\/\/ (\d+)\s+(?=var cssReload)/, '')
          .replaceAll(/var data = "(?:.*)"/g, (match) => {
            return decodeURIComponent(match).replaceAll(/\\/g, '/');
          }),
      );
    };

    const replaceFileName = (str: string) => {
      return Object.entries({
        ...resultHashes,
        ...runtimes,
      }).reduce((str, [raw, replacement]) => {
        return str.split(raw).join(replacement);
      }, str);
    };

    const assets = stats.assets!.sort((a, b) => a.name.localeCompare(b.name));
    const fileList = assets
      .map((i) => {
        const fileName = i.name;
        const renderName = replaceFileName(fileName);
        const content = replaceContent(
          fs.readFileSync(context.getDist(fileName), 'utf-8'),
        );
        if (fileName.endsWith('hot-update.js')) {
          const modules = getModuleHandler(context.getDist(fileName), options);
          const runtime: string[] = [];
          for (const i of content.matchAll(
            /\/\/ (webpack\/runtime\/[\w_-]+)\s*\n/g,
          )) {
            runtime.push(i[1]);
          }
          modules.sort();
          runtime.sort();
          hotUpdateFile.push({
            name: renderName,
            content,
            modules,
            runtime,
          });
          return `- Update: ${renderName}, size: ${content.length}`;
        }
        if (fileName.endsWith('hot-update.json')) {
          const manifest = JSON.parse(content);
          manifest.c?.sort();
          manifest.r?.sort();
          manifest.m?.sort();
          hotUpdateManifest.push({
            name: renderName,
            content: JSON.stringify(manifest),
          });
          return `- Manifest: ${renderName}, size: ${i.size}`;
        }
        if (fileName.endsWith('.js')) {
          return `- Bundle: ${renderName}`;
        }
      })
      .filter(Boolean);

    fileList.sort();
    hotUpdateManifest.sort((a, b) => (a.name > b.name ? 1 : -1));
    hotUpdateFile.sort((a, b) => (a.name > b.name ? 1 : -1));

    if (runtime?.javascript) {
      runtime.javascript.outdatedModules.sort();
      runtime.javascript.updatedModules.sort();
      runtime.javascript.updatedRuntime.sort();
      runtime.javascript.acceptedModules.sort();
      runtime.javascript.disposedModules.sort();
      for (const value of Object.values(
        runtime.javascript.outdatedDependencies,
      )) {
        value.sort();
      }
    }

    const content = `
# ${title}

## Changed Files
${changedFiles.map((i) => `- ${i}`).join('\n')}

## Asset Files
${fileList.join('\n')}

## Manifest
${hotUpdateManifest
  .map(
    (i) => `
### ${i.name}

\`\`\`json
${i.content}
\`\`\`
`,
  )
  .join('\n\n')}

## Update

${hotUpdateFile
  .map(
    (i) => `
### ${i.name}

#### Changed Modules
${i.modules.map((i) => `- ${i}`).join('\n')}

#### Changed Runtime Modules
${i.runtime.map((i) => `- ${i}`).join('\n')}

#### Changed Content
\`\`\`js
${i.content}
\`\`\`
`,
  )
  .join('\n\n')}


${
  runtime
    ? `
## Runtime
### Status

\`\`\`txt
${runtime.statusPath.join(' => ')}
\`\`\`

${
  runtime.javascript
    ? `

### JavaScript

#### Outdated

Outdated Modules:
${runtime.javascript.outdatedModules.map((i) => `- ${i}`).join('\n')}


Outdated Dependencies:
\`\`\`json
${JSON.stringify(runtime.javascript.outdatedDependencies || {}, null, 2)}
\`\`\`

#### Updated

Updated Modules:
${runtime.javascript.updatedModules.map((i) => `- ${i}`).join('\n')}

Updated Runtime:
${runtime.javascript.updatedRuntime.map((i) => `- \`${i}\``).join('\n')}


#### Callback

Accepted Callback:
${runtime.javascript.acceptedModules.map((i) => `- ${i}`).join('\n')}

Disposed Callback:
${runtime.javascript.disposedModules.map((i) => `- ${i}`).join('\n')}
`
    : ''
}

`
    : ''
}

				`
      .replaceAll(/%3A(\d+)%2F/g, (match, capture) => {
        return match.replace(capture, 'PORT');
      })
      .trim();

    env.expect(content).toMatchFileSnapshotSync(snapshotPath);
  }

  const originRun = processor.run;
  processor.run = async function (env, context) {
    context.setValue(
      'hotUpdateStepChecker',
      (updateIndex: number, stats: Stats, runtime: THotStepRuntimeData) => {
        const statsJson: StatsCompilation = stats.toJson({
          assets: true,
          chunks: true,
        });

        const chunks = Array.from(
          // Some chunk fields are missing from rspack
          (stats?.compilation.chunks as unknown as Chunk[]) || NOOP_SET,
        );

        for (const entry of chunks.filter((i) => i.hasRuntime())) {
          if (!entries[entry.id!] && entry.runtime) {
            entries[entry.id!] =
              // Webpack uses `string | SortableSet<string>` for `entry.runtime`
              typeof entry.runtime === 'string'
                ? [entry.runtime]
                : Array.from(entry.runtime);
          }
        }
        const compiler = context.getCompiler();
        const compilerOptions = compiler.getOptions();
        matchStepSnapshot(
          env,
          context,
          updateIndex,
          compilerOptions,
          statsJson,
          runtime,
        );
        hashes.push(stats.hash!);
      },
    );
    context.setValue(
      'hotUpdateStepErrorChecker',
      (updateIndex: number, stats: Stats, runtime: THotStepRuntimeData) => {
        hashes.push(stats.hash!);
      },
    );

    await originRun(env, context);
  };

  processor.check = async function (env, context) {
    const compiler = context.getCompiler();
    const stats = compiler.getStats() as Stats;
    if (!stats || !stats.hash) {
      env.expect(false);
      return;
    }
    const statsJson = stats.toJson({ assets: true, chunks: true });

    const chunks = Array.from(
      // Some chunk fields are missing from rspack
      (stats?.compilation.chunks as unknown as Chunk[]) || NOOP_SET,
    );

    for (const entry of chunks.filter((i) => i.hasRuntime())) {
      if (entry.runtime) {
        entries[entry.id!] =
          // Webpack uses `string | SortableSet<string>` for `entry.runtime`
          typeof entry.runtime === 'string'
            ? [entry.runtime]
            : Array.from(entry.runtime);
      }
    }
    let matchFailed: Error | null = null;
    try {
      matchStepSnapshot(env, context, 0, compiler.getOptions(), statsJson);
    } catch (e) {
      matchFailed = e as Error;
    }
    hashes.push(stats.hash!);
    if (matchFailed) {
      throw matchFailed;
    }
  };

  return processor;
}

function getCreator(target: TTarget) {
  if (!creators.has(target)) {
    creators.set(
      target,
      new BasicCaseCreator({
        clean: true,
        describe: false,
        target,
        steps: ({ name, target, src, temp, dist }) => [
          createHotStepProcessor(
            name,
            src,
            temp || path.resolve(dist, 'temp'),
            target as TTarget,
          ),
        ],
        runner: {
          key: (context: ITestContext, name: string, file: string) => name,
          runner: createHotRunner,
        },
        concurrent: true,
      }),
    );
  }
  return creators.get(target)!;
}

export function createHotStepCase(
  name: string,
  src: string,
  dist: string,
  temp: string,
  target: RspackOptions['target'],
) {
  const creator = getCreator(target);
  creator.create(name, src, dist, temp);
}
