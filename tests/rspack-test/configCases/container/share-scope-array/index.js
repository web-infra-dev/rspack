import { container } from '@rspack/core';

it('should initialize a single-item shareScope array', async () => {
  const remote = require('./federation.js');
  const scope = {};
  await remote.init(scope);
  const factory = await remote.get('./scope');
  expect(factory()).toBe(scope);
});

it('should reject empty shareScope arrays for containers and remotes', () => {
  for (const enhanced of [false, true]) {
    for (const [Plugin, options] of [
      [container.ContainerPlugin, { name: 'container', exposes: {}, shareScope: [] }],
      [container.ContainerReferencePlugin, {
        remoteType: 'commonjs', remotes: {}, shareScope: [],
      }],
      [container.ContainerReferencePlugin, {
        remoteType: 'commonjs',
        remotes: { remote: { external: './remote', shareScope: [] } },
      }],
    ]) {
      expect(() => new Plugin({ ...options, enhanced })).toThrow(
        `[${Plugin.name}] shareScope array must contain at least one entry`,
      );
    }
  }
});

it('should reject multiple share scopes without the enhanced runtime', () => {
  expect(
    () =>
      new container.ContainerPlugin({
        name: 'container',
        exposes: {},
        shareScope: ['custom', 'other'],
      }),
  ).toThrow('shareScope as an array with multiple entries requires enhanced=true');
});
