import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadConfig as baseLoadConfig } from '@rstackjs/load-config';
import type { MultiRspackOptions, RspackOptions } from '@rspack/core';
import findConfig from './findConfig';
import type { CommonOptions } from './options';

const require = createRequire(import.meta.url);

const DEFAULT_CONFIG_NAME = 'rspack.config' as const;

export type LoadedRspackConfig = RspackOptions | MultiRspackOptions;

type ConfigParams = [
  Record<string, unknown> | string[] | undefined,
  CommonOptions,
];

const loadConfigByPath = async (
  configPath: string,
  options: CommonOptions,
): Promise<LoadedRspackConfig> => {
  const configParams: ConfigParams = [options.env, options];

  const { content } = await baseLoadConfig<LoadedRspackConfig, ConfigParams>({
    path: configPath,
    loader: options.configLoader,
    configParams,
    fresh: true,
  });

  if (!isRspackConfig(content)) {
    throw new Error(
      `[rspack-cli:loadConfig] The config at "${configPath}" must be an object or an array, got ${String(
        content,
      )}`,
    );
  }

  return content;
};

const isConfigObject = (value: unknown): value is Record<string, unknown> =>
  Boolean(value) && typeof value === 'object' && !Array.isArray(value);

const isRspackConfig = (
  value: unknown,
): value is RspackOptions | MultiRspackOptions =>
  Array.isArray(value) || isConfigObject(value);

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
  visitedPaths?: Set<string>,
): Promise<{
  config: RspackOptions;
  pathMap: WeakMap<RspackOptions, string[]>;
}>;
export async function loadExtendedConfig(
  config: MultiRspackOptions,
  configPath: string,
  cwd: string,
  options: CommonOptions,
  visitedPaths?: Set<string>,
): Promise<{
  config: MultiRspackOptions;
  pathMap: WeakMap<RspackOptions, string[]>;
}>;
export async function loadExtendedConfig(
  config: RspackOptions | MultiRspackOptions,
  configPath: string,
  cwd: string,
  options: CommonOptions,
  visitedPaths?: Set<string>,
): Promise<{
  config: RspackOptions | MultiRspackOptions;
  pathMap: WeakMap<RspackOptions, string[]>;
}>;
export async function loadExtendedConfig(
  config: RspackOptions | MultiRspackOptions,
  configPath: string,
  cwd: string,
  options: CommonOptions,
  visitedPaths?: Set<string>,
): Promise<{
  config: RspackOptions | MultiRspackOptions;
  pathMap: WeakMap<RspackOptions, string[]>;
}> {
  const currentVisitedPaths = visitedPaths ?? new Set<string>();

  if (checkIsMultiRspackOptions(config)) {
    // If the config is an array, we need to handle each item separately
    const resultPathMap = new WeakMap();
    const extendedConfigs = (await Promise.all(
      config.map(async (item) => {
        const itemVisitedPaths = new Set(currentVisitedPaths);
        const { config, pathMap } = await loadExtendedConfig(
          item,
          configPath,
          cwd,
          options,
          itemVisitedPaths,
        );
        resultPathMap.set(config, pathMap.get(config));
        return config;
      }),
    )) as MultiRspackOptions;
    extendedConfigs.parallelism = config.parallelism;
    return { config: extendedConfigs, pathMap: resultPathMap };
  }

  if (currentVisitedPaths.has(configPath)) {
    throw new Error(
      `Recursive configuration detected. Config file "${configPath}" extends itself.`,
    );
  }
  currentVisitedPaths.add(configPath);
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

    if (extendPath.startsWith('file://')) {
      try {
        resolvedPath = fileURLToPath(extendPath);
      } catch {
        throw new Error(
          `Invalid file URL '${extendPath}' in extends configuration.`,
        );
      }
    }
    // Check if it's a node module or a relative path
    else if (
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

    // Load the extended configuration
    const loadedConfig = await loadConfigByPath(resolvedPath, options);
    const { merge } = await import('rspack-merge');

    // Recursively load extended configurations from the extended config
    const { config: extendedConfig, pathMap: extendedPathMap } =
      (await loadExtendedConfig(
        loadedConfig,
        resolvedPath,
        cwd,
        options,
        currentVisitedPaths,
      )) as {
        config: RspackOptions;
        pathMap: WeakMap<RspackOptions, string[]>;
      };
    // Calc config paths
    const configPaths = [
      ...(pathMap.get(resultConfig) || []),
      ...(extendedPathMap.get(extendedConfig) || []),
    ];
    // Merge the configurations
    resultConfig = merge(extendedConfig, resultConfig);
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
  const loadedConfig = await loadConfigByPath(configPath, options);

  return { loadedConfig, configPath };
}
