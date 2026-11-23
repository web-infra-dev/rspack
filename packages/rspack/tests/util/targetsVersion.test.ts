import { describe, expect, it } from 'rstack/test';
import { decodeVersion, encodeVersion } from '../../src/util/targetsVersion';

describe('targetsVersion', () => {
  it('should round-trip valid browser versions', () => {
    expect(decodeVersion(encodeVersion('120')!)).toBe('120.0.0');
    expect(decodeVersion(encodeVersion('16.4-16.6')!)).toBe('16.4.0');
    expect(decodeVersion(encodeVersion('1.2.3')!)).toBe('1.2.3');
  });

  it('should clamp minor and patch components exceeding 255 without overflowing into higher octets', () => {
    expect(decodeVersion(encodeVersion('145.0.7632.45')!)).toBe('145.0.255');
    expect(decodeVersion(encodeVersion('1.256.0')!)).toBe('1.255.0');
    expect(decodeVersion(encodeVersion('1.0.256')!)).toBe('1.0.255');
  });

  it('should return null for out-of-range major or negative version components', () => {
    expect(encodeVersion('65536.0.0')).toBeNull();
    expect(encodeVersion('-1.0.0')).toBeNull();
    expect(encodeVersion('1.-1.0')).toBeNull();
    expect(encodeVersion('1.0.-1')).toBeNull();
    expect(encodeVersion('invalid')).toBeNull();
  });
});
