import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import type { MultiRspackOptions, RspackOptions } from '@rspack/core';
import { rspack } from '@rspack/core';
import findConfig from './findConfig';
import isTsFile from './isTsFile';
import type { CommonOptions } from './options';

const require = createRequire(import.meta.url);

const DEFAULT_CONFIG_NAME = 'rspack.config' as const;

const JS_CONFIG_EXTENSION_REGEXP = /\.(?:js|mjs|cjs)$/;
const CONFIG_LOADER_VALUES = ['auto', 'jiti', 'native'] as const;
type ConfigLoader = (typeof CONFIG_LOADER_VALUES)[number];

const readPackageUp = (cwd = process.cwd()): { type?: 'module' } | null => {
  let currentDir = cwd;

  while (currentDir !== path.dirname(currentDir)) {
    const packagePath = path.join(currentDir, 'package.json');
    if (fs.existsSync(packagePath)) {
      return JSON.parse(fs.readFileSync(packagePath, 'utf8'));
    }
    currentDir = path.dirname(currentDir);
  }

  return null;
};

const isEsmFile = (filePath: string) => {
  if (/\.(mjs|mts)$/.test(filePath)) {
    return true;
  }
  if (/\.(cjs|cts)$/.test(filePath)) {
    return false;
  }

  const packageJson = readPackageUp(path.dirname(filePath));
  return packageJson?.type === 'module';
};

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

const loadConfigWithRequire = <T = unknown>(configPath: string): T =>
  resolveDefaultExport(require(configPath) as T);

const hasTypeScriptRequireLoader = () =>
  Boolean(require.extensions['.ts'] || require.extensions['.cts']);

let jitiInstancePromise:
  | Promise<ReturnType<(typeof import('jiti'))['createJiti']>>
  | undefined;

const getJiti = async () => {
  if (!jitiInstancePromise) {
    jitiInstancePromise = import('jiti').then(({ createJiti }) =>
      createJiti(import.meta.url, {
        moduleCache: false,
        interopDefault: true,
        nativeModules: ['@rspack/cli', '@rspack/core', 'typescript'],
      }),
    );
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
  const isTypeScriptConfig = isTsFile(configPath);
  const isTypeScriptEsmConfig = isTypeScriptConfig && isEsmFile(configPath);
  const shouldTryRequireLoader =
    isTypeScriptConfig &&
    !isTypeScriptEsmConfig &&
    configLoader !== 'jiti' &&
    hasTypeScriptRequireLoader();
  const shouldTryNativeLoader =
    configLoader === 'native' ||
    JS_CONFIG_EXTENSION_REGEXP.test(configPath) ||
    (configLoader === 'auto' &&
      supportsNativeTypeScript() &&
      isTypeScriptEsmConfig);

  if (shouldTryRequireLoader) {
    return loadConfigWithRequire<T>(configPath);
  }

  if (shouldTryNativeLoader) {
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

    // Load the extended configuration
    let loadedConfig = await loadConfigByPath<LoadedRspackConfig>(
      resolvedPath,
      options,
    );

    // If the extended config is a function, execute it
    if (typeof loadedConfig === 'function') {
      loadedConfig = loadedConfig(options.env as Record<string, any>, options);
      // if return promise we should await its result
      if (
        typeof (loadedConfig as unknown as Promise<unknown>).then === 'function'
      ) {
        loadedConfig = await loadedConfig;
      }
    }

    // Recursively load extended configurations from the extended config
    const { config: extendedConfig, pathMap: extendedPathMap } =
      (await loadExtendedConfig(
        loadedConfig as RspackOptions | MultiRspackOptions,
        resolvedPath,
        cwd,
        options,
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
  const loadedConfig = await loadConfigByPath<LoadedRspackConfig>(
    configPath,
    options,
  );

  return { loadedConfig, configPath };
}
