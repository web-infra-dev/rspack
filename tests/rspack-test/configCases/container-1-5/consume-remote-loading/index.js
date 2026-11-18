it('should preserve the consume strategy before and after importing a commonjs remote', async () => {
  try {
    const { default: value } = await import('lib');
    expect(value).toBe(__EXPECT_REMOTE_PROVIDER__ ? 'remote' : 'local');
    expect(globalThis.__mfConsumeRemoteLoads?.[__dirname] || 0).toBe(__EXPECT_REMOTE_PROVIDER__ ? 1 : 0);
    expect((await import('remote/value')).default).toBe('initialized');
    expect(globalThis.__mfConsumeRemoteLoads[__dirname]).toBe(1);
  } finally {
    delete globalThis.__mfConsumeRemoteLoads?.[__dirname];
  }
});

// Keep the pending promise remote in the scalar build without requesting it.
if (__HAS_PENDING_REMOTE__) {
  module.exports.loadPending = () => import('pending/value');
}
