// Forwarding public options must not require casts or a required enhanced flag.
import rspack, {
  type ConsumeSharedPluginOptions,
  type ContainerPluginOptions,
  type ModuleFederationPluginOptions,
  type ModuleFederationPluginV1Options,
  type Exposes,
} from '@rspack/core';

// Annotated public options can opt in to the enhanced runtime.
const a: ConsumeSharedPluginOptions = {
  consumes: { react: {} },
  enhanced: true,
};
new rspack.sharing.ConsumeSharedPlugin(a);

const b: ContainerPluginOptions = {
  name: 'b',
  exposes: { './x': './x' },
  enhanced: true,
};
new rspack.container.ContainerPlugin(b);

// Wrappers can forward legacy V1 options unchanged.
export function makeV1(
  o: ModuleFederationPluginV1Options,
): InstanceType<typeof rspack.container.ModuleFederationPluginV1> {
  return new rspack.container.ModuleFederationPluginV1(o);
}

// MF 1.5 options can be forwarded to the V1 plugin with enhanced: true.
export function toV1(
  o: ModuleFederationPluginOptions,
): ModuleFederationPluginV1Options {
  return {
    name: o.name,
    exposes: o.exposes,
    remotes: o.remotes,
    shared: o.shared,
    enhanced: true,
  };
}

// A legacy exposes map is also accepted by the enhanced plugin.
const exposes: Exposes = { './x': { import: './x' } };
new rspack.container.ModuleFederationPlugin({ name: 'e', exposes });

// Subclasses can forward options with an optional enhanced flag.
export class MyConsume extends rspack.sharing.ConsumeSharedPlugin {
  constructor(o: ConsumeSharedPluginOptions) {
    super(o);
  }
}

// A dynamic feature flag does not require a different public options type.
declare const enhanced: boolean;
new rspack.container.ContainerPlugin({
  name: 'g',
  exposes: { './x': './x' },
  enhanced,
});
