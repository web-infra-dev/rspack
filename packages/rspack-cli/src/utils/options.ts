import type { Command } from 'cac';

/**
 * Apply common options for all commands
 */
export const commonOptions = (command: Command): Command =>
  command
    .option('-c, --config <path>', 'config file')
    .option('--config-name <name>', 'Name(s) of the configuration to use.', {
      type: [String],
      default: [],
    })
    .option(
      '--config-loader <loader>',
      'Specify the loader to load the config file, can be `native` or `register`.',
      { default: 'register' },
    )
    .option('--env <env>', 'env passed to config function', {
      type: [String],
      default: [],
    })
    .option(
      '--node-env <value>',
      'sets `process.env.NODE_ENV` to be specified value',
    );

export type CommonOptions = {
  config?: string;
  configName?: string[];
  configLoader?: string;
  env?: Record<string, unknown> | string[];
  nodeEnv?: string;
};

function normalizeDevtoolOption(
  value?: string | boolean,
): string | boolean | undefined {
  if (typeof value === 'string') {
    const trimmed = value.trim();
    if (trimmed === '' || trimmed === 'false') {
      return false;
    }
    if (trimmed === 'true') {
      return 'source-map';
    }
    return trimmed;
  }
  if (typeof value === 'boolean') {
    return value ? 'source-map' : false;
  }
}

export const normalizeCommonOptions = (
  options: CommonOptions | CommonOptionsForBuildAndServe,
  action: 'serve' | 'build' | 'preview',
) => {
  const isEmptyArray = (arr?: unknown[]): arr is [] =>
    Array.isArray(arr) && arr.length === 0;

  // remove empty array
  for (const key of ['entry', 'configName'] as const) {
    const val = (options as CommonOptionsForBuildAndServe)[key];
    if (isEmptyArray(val)) {
      (options as CommonOptionsForBuildAndServe)[key] = undefined;
    }
  }

  // normalize options.env
  const env = Array.isArray(options.env) ? normalizeEnvToObject(options) : {};
  options.env = env;
  if (action === 'serve') {
    setBuiltinEnvArg(env, 'SERVE', true);
  } else if (action === 'build') {
    if ((options as CommonOptionsForBuildAndServe).watch) {
      setBuiltinEnvArg(env, 'WATCH', true);
    } else {
      setBuiltinEnvArg(env, 'BUNDLE', true);
      setBuiltinEnvArg(env, 'BUILD', true);
    }
  }

  // normalize options.devtool
  if ('devtool' in options) {
    options.devtool = normalizeDevtoolOption(options.devtool);
  }
};

/**
 * Apply common options for `build` and `serve` commands
 */
export const commonOptionsForBuildAndServe = (command: Command): Command => {
  return command
    .option(
      '-d, --devtool <value>',
      'specify a developer tool for debugging. Defaults to `cheap-module-source-map` in development and `source-map` in production.',
    )
    .option('--entry <entry>', 'entry file', {
      type: [String],
      default: [],
    })
    .option('-m, --mode <mode>', 'mode')
    .option('-o, --output-path <dir>', 'output path dir')
    .option('-w, --watch', 'watch');
};

export type CommonOptionsForBuildAndServe = CommonOptions & {
  devtool?: string | boolean;
  entry?: string[];
  mode?: string;
  outputPath?: string;
  watch?: boolean;
};

/**
 * set builtin env from cli - like `RSPACK_BUNDLE=true`
 * @param env the `argv.env` object
 * @param envNameSuffix the added env will be `RSPACK_${envNameSuffix}`
 * @param value
 */
function setBuiltinEnvArg(
  env: Record<string, any>,
  envNameSuffix: string,
  value: unknown,
) {
  const envName = `RSPACK_${envNameSuffix}`;
  if (!(envName in env)) {
    env[envName] = value;
  }
}

function normalizeEnvToObject(options: CommonOptions) {
  function parseValue(previous: Record<string, unknown>, value: string) {
    const [allKeys, val] = value.split(/=(.+)/, 2);
    const splitKeys = allKeys.split(/\.(?!$)/);

    let prevRef = previous;

    splitKeys.forEach((key, index) => {
      let someKey = key;

      // https://github.com/webpack/webpack-cli/issues/3284
      if (someKey.endsWith('=')) {
        // remove '=' from key
        someKey = someKey.slice(0, -1);
        prevRef[someKey] = undefined;
        return;
      }

      if (!prevRef[someKey] || typeof prevRef[someKey] === 'string') {
        prevRef[someKey] = {};
      }

      if (index === splitKeys.length - 1) {
        if (typeof val === 'string') {
          prevRef[someKey] = val;
        } else {
          prevRef[someKey] = true;
        }
      }

      prevRef = prevRef[someKey] as Record<string, string | object | boolean>;
    });

    return previous;
  }

  return ((options.env as string[]) ?? []).reduce(parseValue, {});
}

export function setDefaultNodeEnv(
  options: { nodeEnv?: unknown },
  defaultEnv: string,
): void {
  if (process.env.NODE_ENV === undefined) {
    process.env.NODE_ENV =
      typeof options.nodeEnv === 'string' ? options.nodeEnv : defaultEnv;
  }
}
