const value = require('lib');

it('should initialize ordered eager consumes without making loaded-first remotes eager', async () => {
  try {
    expect(value).toBe(__EXPECT_REMOTE_PROVIDER__ ? 'remote' : 'local');
    expect(globalThis.__mfConsumeRemoteLoads?.[__dirname] || 0).toBe(__EXPECT_REMOTE_PROVIDER__ ? 1 : 0);
    expect((await import('remote/value')).default).toBe('initialized');
    expect(globalThis.__mfConsumeRemoteLoads[__dirname]).toBe(1);
  } finally {
    delete globalThis.__mfConsumeRemoteLoads?.[__dirname];
  }
});
