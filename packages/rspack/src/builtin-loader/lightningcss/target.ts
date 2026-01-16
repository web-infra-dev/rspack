import type binding from '@rspack/binding';
import { encodeVersion } from '../../util/targetVersion';
import type { Targets } from './index';

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

export const resolveDefaultLightningCssTargets = (
  platforms: string[],
): binding.RawLightningCssBrowsers => {
  const targets: Record<string, number> = {};
  for (const p of platforms) {
    const [n, v] = p.split(' ');
    const remap = REMAP[n];
    if (remap === null) continue;

    const name = remap || n;
    const version = encodeVersion(v);
    if (version === null) continue;

    if (!targets[name] || version < targets[name]) {
      targets[name] = version;
    }
  }
  return targets;
};

export const encodeTargets = (
  targets: Targets,
): binding.RawLightningCssBrowsers => {
  return Object.fromEntries(
    Object.entries(targets).map(([k, v]) => [k, encodeVersion(v)]),
  );
};
