import path from 'node:path';
import {
  Compilation,
  Compiler,
  type RspackOptions,
  sources,
} from '@rspack/core';
import { getSerializers } from 'jest-snapshot';
import { createSnapshotSerializer as createPathSerializer } from 'path-serializer';
import {
  type PrettyFormatOptions,
  format as prettyFormat,
} from 'pretty-format';
import merge from 'webpack-merge';
import { TestContext, type TTestContextOptions } from '../test/context';
import { BasicCaseCreator } from '../test/creator';
import type { ITestContext, ITestEnv, ITesterConfig } from '../type';
import { build, checkSnapshot, compiler, config } from './common';

const srcDir = __TEST_FIXTURES_PATH__;
const distDir = path.resolve(__TEST_DIST_PATH__, 'hook');

const creator = new BasicCaseCreator({
  clean: true,
  describe: true,
  createContext: (config: ITesterConfig) =>
    new HookCasesContext(config.src, config.name, config),
  steps: ({ name, caseConfig: _caseConfig, src }) => {
    const caseConfig = _caseConfig as THookCaseConfig;
    return [
      {
        config: async (context: ITestContext) => {
          const compiler = context.getCompiler();
          const options = await config(
            context,
            name,
            ['rspack.config.js', 'webpack.config.js'],
            defaultOptions(context, caseConfig.options),
          );
          if (!global.printLogger) {
            options.infrastructureLogging = {
              level: 'error',
            };
          }
          compiler.setOptions(options);
        },
        compiler: async (context: ITestContext) => {
          const c = await compiler(context, name);
          if (caseConfig.compiler) {
            await caseConfig.compiler(context, c);
          }
        },
        build: async (context: ITestContext) => {
          await build(context, name);
        },
        run: async (env: ITestEnv, context: ITestContext) => {
          // no need to run, just check snapshot
        },
        check: async (env: ITestEnv, context: ITestContext) => {
          await checkSnapshot(
            env,
            context,
            name,
            path.join(src, 'output.snap.txt'),
            caseConfig.snapshotFileFilter,
          );
        },
      },
    ];
  },
});

export function createHookCase(
  name: string,
  src: string,
  dist: string,
  source: string,
) {
  const caseConfig: Partial<THookCaseConfig> = require(
    path.join(src, 'test.js'),
  );
  const testName = path.basename(
    name.slice(0, name.indexOf(path.extname(name))),
  );
  creator.create(name, src, dist, undefined, {
    caseConfig,
    description: () => caseConfig.description!,
    createContext: (config: ITesterConfig) =>
      new HookCasesContext(src, testName, {
        src: source,
        dist: dist,
        name: name,
      }),
  });
}

const sourceSerializer = {
  test(val: unknown) {
    return val instanceof sources.Source;
  },
  print(val: sources.Source) {
    return val.source();
  },
};

const internalSerializer = {
  test(val: unknown) {
    return val instanceof Compiler || val instanceof Compilation;
  },
  print(val: Compiler | Compilation) {
    return JSON.stringify(`${val.constructor.name}(internal ignored)`);
  },
};

const testPathSerializer = createPathSerializer({
  replace: [
    {
      match: srcDir,
      mark: '<HOOK_SRC_DIR>',
    },
    {
      match: distDir,
      mark: '<HOOK_DIST_DIR>',
    },
  ],
});

const escapeRegex = true;
const printFunctionName = false;
const normalizeNewlines = (str: string) => str.replace(/\r\n|\r/g, '\n');
const serialize = (val: unknown, indent = 2, formatOverrides = {}) =>
  normalizeNewlines(
    prettyFormat(val, {
      escapeRegex,
      indent,
      plugins: [
        ...getSerializers(),
        sourceSerializer,
        internalSerializer,
        testPathSerializer,
      ] as PrettyFormatOptions['plugins'],
      printFunctionName,
      ...formatOverrides,
    }),
  );

export class HookCasesContext extends TestContext {
  protected promises: Promise<void>[] = [];
  protected count = 0;
  protected snapshots: Record<
    string | number,
    Array<[string | Buffer, string]>
  > = {};
  protected snapshotsList: Array<string | number> = [];

  constructor(
    protected src: string,
    protected testName: string,
    protected options: TTestContextOptions,
  ) {
    super(options);
    this.snapped = this.snapped.bind(this);
  }

  /**
   * Snapshot function arguments and return value.
   * Generated snapshot is located in the same directory with the test source.
   * @example
   * compiler.hooks.compilation("name", context.snapped((...args) => { ... }))
   */
  snapped(cb: (...args: unknown[]) => Promise<unknown>, prefix = '') {
    // eslint-disable-next-line
    const context = this;
    return function SNAPPED_HOOK(this: any, ...args: unknown[]) {
      const group = prefix ? prefix : context.count++;
      context._addSnapshot(args, 'input', group);
      const output = cb.apply(this, args);
      if (output && typeof output.then === 'function') {
        let resolve: ((value: void | PromiseLike<void>) => void) | undefined;
        context.promises.push(new Promise((r) => (resolve = r)));
        return output
          .then((o: unknown) => {
            context._addSnapshot(o, 'output (Promise resolved)', group);
            return o;
          })
          .catch((o: unknown) => {
            context._addSnapshot(o, 'output (Promise rejected)', group);
            return o;
          })
          .finally(resolve);
      }
      context._addSnapshot(output, 'output', group);
      return output;
    };
  }

  /**
   * @internal
   */
  _addSnapshot(content: unknown, name: string, group: string | number) {
    const normalizedContent = Buffer.isBuffer(content)
      ? content
      : serialize(content, undefined, {
          escapeString: true,
          printBasicPrototype: true,
        }).replace(/\r\n/g, '\n');

    (this.snapshots[group] = this.snapshots[group] || []).push([
      normalizedContent,
      name,
    ]);
    if (!this.snapshotsList.includes(group)) {
      this.snapshotsList.push(group);
    }
  }

  /**
   * @internal
   */
  async collectSnapshots(
    env: ITestEnv,
    options = {
      diff: {},
    },
  ) {
    await Promise.allSettled(this.promises);
    if (!this.snapshotsList.length) return;

    const snapshots = this.snapshotsList.reduce((acc, group, index) => {
      const block = this.snapshots[group || index].reduce(
        (acc, [content, name]) => {
          name = `## ${name || `test: ${index}`}\n\n`;
          const block = `\`\`\`javascript\n${content}\n\`\`\`\n`;
          return `${acc}${name + block}\n`;
        },
        '',
      );
      return `${acc}# ${Number.isInteger(group) ? `Group: ${index}` : group}\n\n${block}`;
    }, '');
    env
      .expect(snapshots)
      .toMatchFileSnapshotSync(path.join(this.src, 'hooks.snap.txt'), options);
  }
}

export type THookCaseConfig = {
  options?: (context: ITestContext) => RspackOptions;
  compiler?: (context: ITestContext, compiler: Compiler) => Promise<void>;
  check?: (context: ITestContext) => Promise<void>;
  snapshotFileFilter?: (file: string) => boolean;
  description: string;
};

function defaultOptions(
  context: ITestContext,
  custom?: (context: ITestContext) => RspackOptions,
) {
  let defaultOptions = {
    context: context.getSource(),
    mode: 'production',
    target: 'async-node',
    devtool: false,
    cache: false,
    entry: './hook',
    output: {
      path: context.getDist(),
      bundlerInfo: {
        force: false,
      },
    },
    optimization: {
      minimize: false,
    },
    experiments: {
      css: true,
    },
  } as RspackOptions;
  if (custom) {
    defaultOptions = merge(defaultOptions, custom(context));
  }
  return defaultOptions;
}
