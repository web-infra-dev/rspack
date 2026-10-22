import { rspack } from '@rspack/core';

const { ContainerPlugin, ModuleFederationPluginV1, ModuleFederationPlugin } =
  rspack.container;
const error = '[ContainerPlugin] name must be a string.';

it('should validate the name when creating a container directly', () => {
  for (const name of [undefined, null, 123]) {
    expect(() => new ContainerPlugin({ name, exposes: {} })).toThrow(error);
  }
});

for (const Plugin of [ModuleFederationPluginV1, ModuleFederationPlugin]) {
  it(`should require a name when ${Plugin.name} exposes modules`, () => {
    expect(() =>
      rspack({
        mode: 'none',
        plugins: [new Plugin({ exposes: { './module': './index.js' } })],
      }),
    ).toThrow(error);
  });
}
