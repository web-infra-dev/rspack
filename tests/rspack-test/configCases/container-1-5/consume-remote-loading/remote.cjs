globalThis.__mfConsumeRemoteLoads ||= {};
globalThis.__mfConsumeRemoteLoads[__dirname] = (globalThis.__mfConsumeRemoteLoads[__dirname] || 0) + 1;
let initialized = false;
module.exports = {
  async init(scope) {
    // Ordered version-first must await asynchronous provider registration.
    await Promise.resolve();
    initialized = true;
    scope.lib ||= {};
    scope.lib['2.0.0'] = {
      from: 'remote',
      eager: true,
      get: () => () => 'remote',
    };
  },
  get() {
    return Promise.resolve(() => initialized ? 'initialized' : 'not initialized');
  },
};
