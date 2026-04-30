import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import type { MultiRspackOptions, RspackOptions } from '@rspack/core';
import merge from 'rspack-merge';
import findConfig from './findConfig';
import type { CommonOptions } from './options';

const require = createRequire(import.meta.url);

const DEFAULT_CONFIG_NAME = 'rspack.config' as const;

const JS_CONFIG_EXTENSION_REGEXP = /\.(?:js|mjs|cjs)$/;
const CONFIG_LOADER_VALUES = ['auto', 'jiti', 'native'] as const;
type ConfigLoader = (typeof CONFIG_LOADER_VALUES)[number];
type JitiFactory = (
  id: string,
  opts: {
    moduleCache: boolean;
    interopDefault: boolean;
    nativeModules: string[];
  },
) => {
  import<T = unknown>(
    path: string,
    opts: {
      default: boolean;
    },
  ): Promise<T>;
};

const PREBUNDLED_JITI_PATH = new URL(
  '../compiled/jiti/index.js',
  import.meta.url,
).href;

const supportsNativeTypeScript = () => {
  const features = process.features as NodeJS.ProcessFeatures & {
    typescript?: boolean;
  };

  return Boolean(
    features.typescript || process.versions.bun || process.versions.deno,
  );
};

const normalizeConfigLoader = (
  configLoader: CommonOptions['configLoader'],
): ConfigLoader => {
  const normalizedLoader = configLoader ?? 'auto';

  if (CONFIG_LOADER_VALUES.includes(normalizedLoader as ConfigLoader)) {
    return normalizedLoader as ConfigLoader;
  }

  throw new Error(
    `config loader "${normalizedLoader}" is not supported. Expected one of: ${CONFIG_LOADER_VALUES.join(
      ', ',
    )}.`,
  );
};

const resolveDefaultExport = <T>(result: T): T =>
  result &&
  typeof result === 'object' &&
  'default' in (result as Record<string, unknown>)
    ? ((result as Record<string, unknown>).default as T)
    : result;

const loadConfigWithNativeLoader = async <T = unknown>(
  configPath: string,
): Promise<T> => {
  const configFileURL = pathToFileURL(configPath).href;
  const loadedModule = await import(`${configFileURL}?t=${Date.now()}`);
  return resolveDefaultExport(loadedModule as T);
};

let jitiInstancePromise: Promise<ReturnType<JitiFactory>> | undefined;

const getJiti = async () => {
  if (!jitiInstancePromise) {
    jitiInstancePromise = import(
      /* webpackIgnore: true */ PREBUNDLED_JITI_PATH
    ).then((module) => {
      const createJiti =
        'createJiti' in module
          ? (module.createJiti as JitiFactory)
          : (module.default as JitiFactory);

      return createJiti(import.meta.filename, {
        moduleCache: false,
        interopDefault: true,
        nativeModules: ['typescript'],
      });
    });
  }
  return jitiInstancePromise;
};

const loadConfigWithJiti = async <T = unknown>(configPath: string) => {
  const jiti = await getJiti();
  return jiti.import(configPath, { default: true }) as Promise<T>;
};

const loadConfigByPath = async <T = unknown>(
  configPath: string,
  options: CommonOptions,
): Promise<T> => {
  const configLoader = normalizeConfigLoader(options.configLoader);
  const useNative = Boolean(
    configLoader === 'native' ||
    (configLoader === 'auto' && supportsNativeTypeScript()),
  );

  if (useNative || JS_CONFIG_EXTENSION_REGEXP.test(configPath)) {
    try {
      return await loadConfigWithNativeLoader<T>(configPath);
    } catch (error) {
      if (configLoader === 'native') {
        throw error;
      }
    }
  }

  return loadConfigWithJiti<T>(configPath);
};

export type LoadedRspackConfig =
  | undefined
  | RspackOptions
  | MultiRspackOptions
  | ((
      env: Record<string, any>,
      argv?: Record<string, any>,
    ) =>
      | RspackOptions
      | MultiRspackOptions
      | Promise<RspackOptions | MultiRspackOptions>);

const isConfigObject = (value: unknown): value is Record<string, unknown> =>
  Boolean(value) && typeof value === 'object' && !Array.isArray(value);

const isRspackConfig = (
  value: unknown,
): value is RspackOptions | MultiRspackOptions =>
  Array.isArray(value) || isConfigObject(value);

export const resolveRspackConfigExport = async (
  configExport: LoadedRspackConfig,
  options: CommonOptions,
): Promise<RspackOptions | MultiRspackOptions> => {
  let loadedConfig: unknown = configExport;

  if (typeof loadedConfig === 'function') {
    let functionResult = loadedConfig(
      options.env as Record<string, any>,
      options,
    );

    if (typeof (functionResult as Promise<unknown>).then === 'function') {
      functionResult = await functionResult;
    }

    if (functionResult === undefined) {
      throw new Error(
        '[rspack-cli:loadConfig] The config function must return a config object.',
      );
    }

    loadedConfig = functionResult;
  }

  if (!isRspackConfig(loadedConfig)) {
    throw new Error(
      `[rspack-cli:loadConfig] The config must be an object, an array, or a function that returns one, get ${String(
        loadedConfig,
      )}`,
    );
  }

  return loadedConfig;
};

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
    } else if (
      extendPath.startsWith('.') ||
      extendPath.startsWith('/') ||
      extendPath.includes(':\\')
    ) {
      resolvedPath = path.resolve(baseDir, extendPath);

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
    const loadedConfig = await loadConfigByPath<LoadedRspackConfig>(
      resolvedPath,
      options,
    );
    const resolvedConfig = await resolveRspackConfigExport(
      loadedConfig,
      options,
    );

    const { config: extendedConfig, pathMap: extendedPathMap } =
      (await loadExtendedConfig(
        resolvedConfig,
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
  const loadedConfig = await loadConfigByPath<LoadedRspackConfig>(
    configPath,
    options,
  );

  return { loadedConfig, configPath };
}
