require('./layer-marker');

const value = require('lib');
module.exports = value;

it('uses the local eager share without loading an unimported remote', async () => {
  try {
    expect(value).toBe('local');
    expect(globalThis.__mfConsumeRemoteLoads?.[__dirname] || 0).toBe(0);
    expect((await import('remote/value')).default).toBe('initialized');
    expect(globalThis.__mfConsumeRemoteLoads[__dirname]).toBe(1);
  } finally {
    delete globalThis.__mfConsumeRemoteLoads?.[__dirname];
  }
});
