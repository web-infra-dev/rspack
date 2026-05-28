import { describe, expect, it } from '@rstest/core';
import { normalizeCommonOptions } from '../../src/utils/options';

describe('normalizeCommonOptions --env parsing', () => {
  it('builds a nested object from dotted keys', () => {
    const opts: any = { env: ['app.name=demo', 'app.debug=true'] };
    normalizeCommonOptions(opts, 'build');
    expect(opts.env.app.name).toBe('demo');
    expect(opts.env.app.debug).toBe('true');
  });

  it('sets RSPACK_BUILD and RSPACK_BUNDLE on build action', () => {
    const opts: any = { env: [] };
    normalizeCommonOptions(opts, 'build');
    expect(opts.env.RSPACK_BUILD).toBe(true);
    expect(opts.env.RSPACK_BUNDLE).toBe(true);
  });

  it('keeps env as a plain object so config functions can call Object methods', () => {
    const opts: any = { env: ['app.name=demo', 'flag=true'] };
    normalizeCommonOptions(opts, 'build');
    // User configs commonly do `env.hasOwnProperty(...)` or `env instanceof Object`.
    expect(opts.env instanceof Object).toBe(true);
    expect(Object.prototype.hasOwnProperty.call(opts.env, 'flag')).toBe(true);
    expect(opts.env.hasOwnProperty('app')).toBe(true);
  });

  describe('prototype-pollution hardening', () => {
    afterEach(() => {
      // Defensive cleanup so a regression in this file cannot leak pollution
      // into sibling test files in the same worker.
      delete (Object.prototype as any).RSPACK_BUILD;
      delete (Object.prototype as any).polluted;
      delete (Object.prototype as any).injected;
    });

    it('rejects __proto__ in dotted env path without polluting Object.prototype', () => {
      const opts: any = {
        env: ['__proto__.polluted=yes', '__proto__.RSPACK_BUILD=owned'],
      };
      normalizeCommonOptions(opts, 'build');

      expect(({} as any).polluted).toBeUndefined();
      expect(({} as any).RSPACK_BUILD).toBeUndefined();
    });

    it('rejects constructor.prototype.x payloads', () => {
      const opts: any = { env: ['constructor.prototype.injected=1'] };
      normalizeCommonOptions(opts, 'build');
      expect(({} as any).injected).toBeUndefined();
    });

    it('does not allow attacker to spoof reserved RSPACK_BUILD via prototype', () => {
      const opts: any = { env: ['__proto__.RSPACK_BUILD=owned'] };
      normalizeCommonOptions(opts, 'build');
      // Legitimate write must win — not the attacker-controlled "owned" string.
      expect(opts.env.RSPACK_BUILD).toBe(true);
    });
  });
});
