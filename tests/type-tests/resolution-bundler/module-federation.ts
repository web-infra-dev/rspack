import rspack, {
  type ConsumesConfig,
  type ConsumeSharedPluginOptions,
  type ContainerPluginOptions,
  type ExposesConfig,
  type ModuleFederationPluginV1Options,
} from '@rspack/core';

interface ExtendedConsumesConfig extends ConsumesConfig {
  custom?: boolean;
}

const legacyConsume: ExtendedConsumesConfig = {
  import: false,
  custom: true,
};

new rspack.sharing.ConsumeSharedPlugin({
  consumes: { legacyConsume },
});

const annotatedEnhancedConsume: ConsumeSharedPluginOptions = {
  enhanced: true,
  consumes: { react: { request: 'react-server' } },
};
new rspack.sharing.ConsumeSharedPlugin(annotatedEnhancedConsume);

new rspack.sharing.ConsumeSharedPlugin({
  enhanced: true,
  consumes: {
    react: {
      import: false,
      issuerLayer: 'server',
      layer: 'client',
      request: 'react-server',
    },
  },
});

// @ts-expect-error Enhanced consume fields require the runtime feature gate.
new rspack.sharing.ConsumeSharedPlugin<true>({
  consumes: {
    react: { request: 'react-server' },
  },
});

new rspack.container.ContainerPlugin({
  name: 'enhanced',
  enhanced: true,
  exposes: {
    './entry': { import: './index', layer: 'server' },
  },
});

interface ExtendedContainerOptions extends ContainerPluginOptions {
  customRuntimeFlag?: boolean;
}
const extendedContainer: ExtendedContainerOptions = {
  name: 'extended-container',
  exposes: { './entry': { import: './index' } },
  customRuntimeFlag: true,
};
new rspack.container.ContainerPlugin(extendedContainer);

const dynamicEnhanced = Math.random() > 0.5;
const reusableLegacyExpose: ExposesConfig = { import: './index' };
new rspack.container.ContainerPlugin({
  name: 'dynamic-enhanced',
  enhanced: dynamicEnhanced,
  exposes: { './entry': reusableLegacyExpose },
});

// @ts-expect-error Expose layers require the enhanced runtime gate.
new rspack.container.ContainerPlugin({
  name: 'legacy',
  exposes: {
    './entry': { import: './index', layer: 'server' },
  },
});

const legacyV1Options: ModuleFederationPluginV1Options = {
  name: 'legacy-v1',
  // @ts-expect-error Expose layers require the enhanced runtime gate.
  exposes: {
    './entry': { import: './index', layer: 'server' },
  },
};
new rspack.container.ModuleFederationPluginV1(legacyV1Options);

interface ExtendedV1Options extends ModuleFederationPluginV1Options {
  customRuntimeFlag?: boolean;
}
const extendedV1Options: ExtendedV1Options = {
  name: 'extended-v1',
  customRuntimeFlag: true,
};
new rspack.container.ModuleFederationPluginV1(extendedV1Options);

new rspack.container.ModuleFederationPluginV1({
  name: 'legacy-v1-inferred',
  enhanced: false,
  // @ts-expect-error Expose layers require enhanced: true.
  exposes: {
    './entry': { import: './index', layer: 'server' },
  },
});

new rspack.container.ModuleFederationPluginV1({
  name: 'dynamic-v1',
  enhanced: dynamicEnhanced,
  exposes: { './entry': reusableLegacyExpose },
});
