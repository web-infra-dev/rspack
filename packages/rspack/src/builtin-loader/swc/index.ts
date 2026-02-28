import type { GetLoaderOptions } from '../../config/adapterRuleUse';
import { resolveCollectTypeScriptInfo } from './collectTypeScriptInfo';
import { resolvePluginImport } from './pluginImport';

export type { CollectTypeScriptInfoOptions } from './collectTypeScriptInfo';
export type { PluginImportOptions } from './pluginImport';
export type {
  SwcLoaderEnvConfig,
  SwcLoaderEsParserConfig,
  SwcLoaderJscConfig,
  SwcLoaderModuleConfig,
  SwcLoaderOptions,
  SwcLoaderParserConfig,
  SwcLoaderTransformConfig,
  SwcLoaderTsParserConfig,
} from './types';

function defaultTargetFromRspackTargets(targets: Record<string, string>) {
  const REMAP: Record<string, string | null> = {
    and_chr: 'chrome',
    and_ff: 'firefox',
    ie_mob: 'ie',
    ios_saf: 'ios',
    op_mob: 'opera',
    and_qq: null,
    and_uc: null,
    baidu: null,
    bb: null,
    kaios: null,
    op_mini: null,
  };
  const result: Record<string, string> = {};
  for (const [k, version] of Object.entries(targets)) {
    const remap = REMAP[k];
    if (remap === null) continue;
    const name = remap || k;
    result[name] = version;
  }
  return result;
}

export const getSwcLoaderOptions: GetLoaderOptions = (o, composeOptions) => {
  const options = o ?? {};
  if (typeof options === 'object') {
    // enable `disableAllLints` by default to reduce performance overhead
    options.jsc ??= {};
    options.jsc.experimental ??= {};
    options.jsc.experimental.disableAllLints ??= true;

    // Default target derived from rspack target
    if (
      options.env?.targets === undefined &&
      options.jsc?.target === undefined
    ) {
      if (composeOptions.compiler.target?.targets) {
        options.env ??= {};
        options.env.targets ??= defaultTargetFromRspackTargets(
          composeOptions.compiler.target.targets,
        );
      } else if (composeOptions.compiler.target?.esVersion) {
        const { esVersion } = composeOptions.compiler.target;
        options.jsc.target ??= esVersion >= 2015 ? `es${esVersion}` : 'es5';
      }
    }

    // resolve top-level `collectTypeScriptInfo` options (stable API)
    if (options.collectTypeScriptInfo) {
      options.collectTypeScriptInfo = resolveCollectTypeScriptInfo(
        options.collectTypeScriptInfo,
      );
    }

    // resolve `rspackExperiments.import` options
    const { rspackExperiments } = options;
    if (rspackExperiments) {
      if (rspackExperiments.import || rspackExperiments.pluginImport) {
        rspackExperiments.import = resolvePluginImport(
          rspackExperiments.import || rspackExperiments.pluginImport,
        );
      }
    }
  }
  return options;
};
