import {
  decodeVersion,
  encodeVersion,
} from '../../../../../packages/rspack/src/util/targetsVersion.ts';

it('should preserve Chrome versions with a major version above 255', () => {
  for (const chrome of ['255.1.2', '256.1.2', '257.1.2']) {
    expect(decodeVersion(encodeVersion(chrome))).toBe(chrome);
  }
});
