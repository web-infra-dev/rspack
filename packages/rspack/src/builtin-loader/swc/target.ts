import { decodeVersion, encodeVersion } from '../../util/targetVersion';

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

export const resolveDefaultSwcTargets = (platforms: string[]) => {
  const targets: Record<string, number> = {};
  for (const p of platforms) {
    const [n, v] = p.split(' ');
    const remap = REMAP[n];
    if (remap === null) {
      continue;
    }
    const name = remap || n;
    const version = encodeVersion(v);
    if (version === null) continue;

    if (!targets[name] || version < targets[name]) {
      targets[name] = version;
    }
  }
  return Object.fromEntries(
    Object.entries(targets).map(([k, v]) => [k, decodeVersion(v)]),
  );
};
