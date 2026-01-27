import fs from 'node:fs';
import path from 'node:path';
import type { MultiRspackOptions, RspackOptions } from '@rspack/core';
import { addHook } from 'pirates';
import { crossImport } from './crossImport';
import findConfig from './findConfig';
import { isEsmFile } from './isEsmFile';
import isTsFile, { TS_EXTENSION } from './isTsFile';
import type { CommonOptions } from './options';
import { rspack } from './rspackCore';

const injectInlineSourceMap = ({
  code,
  map,
}: {
  code: string;
  map: string | undefined;
}): string => {
  if (map) {
    const base64Map = Buffer.from(map, 'utf8').toString('base64');
    const sourceMapContent = `//# sourceMappingURL=data:application/json;charset=utf-8;base64,${base64Map}`;
    return `${code}\n${sourceMapContent}`;
  }
  return code;
};

export function compile(sourcecode: string, filename: string) {
  const { code, map } = rspack.experiments.swc.transformSync(sourcecode, {
    jsc: {
      parser: {
        syntax: 'typescript',
        tsx: false,
        decorators: true,
        dynamicImport: true,
      },
    },
    filename: filename,
    module: { type: 'commonjs' },
    sourceMaps: true,
    isModule: true,
  });
  return injectInlineSourceMap({ code, map });
}

const DEFAULT_CONFIG_NAME = 'rspack.config' as const;

// modified based on https://github.com/swc-project/swc-node/blob/master/packages/register/register.ts#L117
const registerLoader = (configPath: string) => {
  // For ESM and `.mts` you need to use: 'NODE_OPTIONS="--loader ts-node/esm" rspack build --config ./rspack.config.mts'
  if (isEsmFile(configPath) && isTsFile(configPath)) {
    return;
  }

  // Only support TypeScript files with a CommonJS loader here
  if (!isTsFile(configPath)) {
    throw new Error(`config file "${configPath}" is not supported.`);
  }

  addHook(
    (code, filename) => {
      try {
        return compile(code, filename);
      } catch (err) {
        throw new Error(
          `Failed to transform file "${filename}" when loading TypeScript config file:\n ${err instanceof Error ? err.message : String(err)}`,
        );
      }
    },
    {
      exts: TS_EXTENSION,
    },
  );
};

export type LoadedRspackConfig =
  | undefined
  | RspackOptions
  | MultiRspackOptions
  | ((
      env: Record<string, any>,
      argv?: Record<string, any>,
    ) => RspackOptions | MultiRspackOptions);

const checkIsMultiRspackOptions = (
  config: RspackOptions | MultiRspackOptions,
): config is MultiRspackOptions => Array.isArray(config);

/**
 * Loads and merges configurations from the 'extends' property
 * @param config The configuration object that may contain an 'extends' property
 * @param configPath The path to the configuration file
 * @param cwd The current working directory
 * @param options CLI options
 * @returns The merged configuration
 */
export async function loadExtendedConfig(
  config: RspackOptions,
  configPath: string,
  cwd: string,
  options: CommonOptions,
): Promise<{
  config: RspackOptions;
  pathMap: WeakMap<RspackOptions, string[]>;
}>;
export async function loadExtendedConfig(
  config: MultiRspackOptions,
  configPath: string,
  cwd: string,
  options: CommonOptions,
): Promise<{
  config: MultiRspackOptions;
  pathMap: WeakMap<RspackOptions, string[]>;
}>;
export async function loadExtendedConfig(
  config: RspackOptions | MultiRspackOptions,
  configPath: string,
  cwd: string,
  options: CommonOptions,
): Promise<{
  config: RspackOptions | MultiRspackOptions;
  pathMap: WeakMap<RspackOptions, string[]>;
}>;
export async function loadExtendedConfig(
  config: RspackOptions | MultiRspackOptions,
  configPath: string,
  cwd: string,
  options: CommonOptions,
): Promise<{
  config: RspackOptions | MultiRspackOptions;
  pathMap: WeakMap<RspackOptions, string[]>;
}> {
  if (checkIsMultiRspackOptions(config)) {
    // If the config is an array, we need to handle each item separately
    const resultPathMap = new WeakMap();
    const extendedConfigs = (await Promise.all(
      config.map(async (item) => {
        const { config, pathMap } = await loadExtendedConfig(
          item,
          configPath,
          cwd,
          options,
        );
        resultPathMap.set(config, pathMap.get(config));
        return config;
      }),
    )) as MultiRspackOptions;
    extendedConfigs.parallelism = config.parallelism;
    return { config: extendedConfigs, pathMap: resultPathMap };
  }
  // set config path
  const pathMap: WeakMap<RspackOptions, string[]> = new WeakMap();
  pathMap.set(config, [configPath]);
  // If there's no extends property, return the config as is
  if (!('extends' in config) || !config.extends) {
    return { config, pathMap };
  }

  // Convert extends to an array if it's a string
  const extendsList = Array.isArray(config.extends)
    ? config.extends
    : [config.extends];

  // Remove the extends property to avoid infinite recursion
  const { extends: _, ...configWithoutExtends } = config as RspackOptions;

  // The base directory for resolving relative paths is the directory of the config file
  const baseDir = path.dirname(configPath);

  // Load and merge configurations from right to left
  let resultConfig = configWithoutExtends;
  pathMap.set(resultConfig, [configPath]);

  for (const extendPath of extendsList) {
    let resolvedPath: string;

    // Check if it's a node module or a relative path
    if (
      extendPath.startsWith('.') ||
      extendPath.startsWith('/') ||
      extendPath.includes(':\\')
    ) {
      // It's a relative or absolute path
      resolvedPath = path.resolve(baseDir, extendPath);

      // If the path doesn't have an extension, try to find a matching config file
      if (!path.extname(resolvedPath)) {
        const foundConfig = findConfig(resolvedPath);
        if (foundConfig) {
          resolvedPath = foundConfig;
        } else {
          throw new Error(
            `Extended configuration file "${resolvedPath}" not found.`,
          );
        }
      }
    } else {
      // It's a node module
      try {
        resolvedPath = require.resolve(extendPath, { paths: [baseDir, cwd] });
      } catch {
        throw new Error(`Cannot find module '${extendPath}' to extend from.`);
      }
    }

    // Check if the file exists
    if (!fs.existsSync(resolvedPath)) {
      throw new Error(
        `Extended configuration file "${resolvedPath}" not found.`,
      );
    }

    // Register loader for TypeScript files
    if (isTsFile(resolvedPath) && options.configLoader === 'register') {
      registerLoader(resolvedPath);
    }

    // Load the extended configuration
    let loadedConfig = await crossImport(resolvedPath);

    // If the extended config is a function, execute it
    if (typeof loadedConfig === 'function') {
      loadedConfig = loadedConfig(options.env, options);
      // if return promise we should await its result
      if (
        typeof (loadedConfig as unknown as Promise<unknown>).then === 'function'
      ) {
        loadedConfig = await loadedConfig;
      }
    }

    // Recursively load extended configurations from the extended config
    const { config: extendedConfig, pathMap: extendedPathMap } =
      await loadExtendedConfig(loadedConfig, resolvedPath, cwd, options);
    // Calc config paths
    const configPaths = [
      ...(pathMap.get(resultConfig) || []),
      ...(extendedPathMap.get(extendedConfig) || []),
    ];
    // Merge the configurations
    resultConfig = rspack.util.cleverMerge(extendedConfig, resultConfig);
    // Set config paths
    pathMap.set(resultConfig, configPaths);
  }

  return { config: resultConfig, pathMap };
}

export async function loadRspackConfig(
  options: CommonOptions,
  cwd = process.cwd(),
): Promise<{ loadedConfig: LoadedRspackConfig; configPath: string } | null> {
  // calc config path.
  let configPath = '';

  if (options.config) {
    configPath = path.resolve(cwd, options.config);
    if (!fs.existsSync(configPath)) {
      throw new Error(`config file "${configPath}" not found.`);
    }
  } else {
    const defaultConfig = findConfig(path.resolve(cwd, DEFAULT_CONFIG_NAME));
    if (!defaultConfig) {
      return null;
    }

    configPath = defaultConfig;
  }

  // load config
  if (isTsFile(configPath) && options.configLoader === 'register') {
    registerLoader(configPath);
  }
  const loadedConfig = await crossImport(configPath);

  return { loadedConfig, configPath };
}
