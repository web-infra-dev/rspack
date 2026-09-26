import value from './value';
import packageValue from './package';

it('should reuse resolutions and invalidate file and missing dependencies', async () => {
  if (COMPILER_INDEX === 0) {
    expect(value).toBe('fallback');
    expect(packageValue).toBe('a');
    await NEXT_HMR();
    expect(value).toBe('fallback');
    expect(packageValue).toBe('a');
    await NEXT_START();
  } else if (COMPILER_INDEX === 1) {
    expect(value).toBe('preferred');
    expect(packageValue).toBe('a');
    await NEXT_START();
  } else if (COMPILER_INDEX === 2) {
    expect(value).toBe('fallback');
    expect(packageValue).toBe('a');
    await NEXT_START();
  } else {
    expect(value).toBe('fallback');
    expect(packageValue).toBe('b');
    if (COMPILER_INDEX === 3) await NEXT_START();
  }
});

module.hot.accept(['./value', './package']);
