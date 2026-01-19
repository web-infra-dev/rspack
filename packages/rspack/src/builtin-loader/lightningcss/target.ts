import type binding from '@rspack/binding';
import { encodeVersion } from '../../util/targetsVersion';
import type { Targets } from './index';

export function encodeTargets(
  targets: Targets,
): binding.RawLightningCssBrowsers {
  return Object.fromEntries(
    Object.entries(targets).map(([k, v]) => [k, encodeVersion(v)]),
  );
}

export function defaultTargetsFromRspackTargets(
  targets: Record<string, string>,
): binding.RawLightningCssBrowsers {
  const REMAP: Record<string, string | null> = {
    and_chr: 'chrome',
    and_ff: 'firefox',
    ie_mob: 'ie',
    op_mob: 'opera',
    and_qq: null,
    and_uc: null,
    baidu: null,
    bb: null,
    kaios: null,
    op_mini: null,
  };
  const result: Record<string, number> = {};
  for (const [k, v] of Object.entries(targets)) {
    const name = REMAP[k] ?? k;
    if (name === null) continue;
    const version = encodeVersion(v);
    if (version === null) continue;
    result[name] = version;
  }
  return result;
}
