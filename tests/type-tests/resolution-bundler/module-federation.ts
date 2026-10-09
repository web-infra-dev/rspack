import rspack, {
  type ConsumesConfig,
  type ConsumeSharedPluginOptions,
  type ContainerPluginOptions,
  type EnhancedConsumeSharedPluginOptions,
  type EnhancedContainerPluginOptions,
  type EnhancedModuleFederationPluginV1Options,
  type ExposesConfig,
  type ExposesObject,
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

// The public options type stays extendable and legacy-compatible.
interface ExtendedConsumeOptions extends ConsumeSharedPluginOptions {
  custom?: boolean;
}
const extendedConsume: ExtendedConsumeOptions = {
  consumes: { legacyConsume },
  custom: true,
};
new rspack.sharing.ConsumeSharedPlugin(extendedConsume);

const annotatedEnhancedConsume: EnhancedConsumeSharedPluginOptions = {
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

const dynamicEnhanced = Math.random() > 0.5;
new rspack.sharing.ConsumeSharedPlugin({
  enhanced: dynamicEnhanced,
  consumes: { legacyConsume },
});

// @ts-expect-error Enhanced consume fields require the runtime feature gate.
new rspack.sharing.ConsumeSharedPlugin<true>({
  consumes: {
    react: { request: 'react-server' },
  },
});

// @ts-expect-error Enhanced consume fields require enhanced: true.
new rspack.sharing.ConsumeSharedPlugin({
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

const reusableEnhancedContainer: EnhancedContainerPluginOptions = {
  name: 'reusable-enhanced-container',
  enhanced: true,
  exposes: {
    './entry': { import: './index', layer: 'server' },
  },
};
new rspack.container.ContainerPlugin(reusableEnhancedContainer);

interface ExtendedContainerOptions extends ContainerPluginOptions {
  customRuntimeFlag?: boolean;
}
const extendedContainer: ExtendedContainerOptions = {
  name: 'extended-container',
  exposes: { './entry': { import: './index' } },
  customRuntimeFlag: true,
};
new rspack.container.ContainerPlugin(extendedContainer);

const reusableLegacyExpose: ExposesConfig = { import: './index' };
const reusableLayeredExposes: ExposesObject<true> = {
  './entry': { import: './index', layer: 'server' },
};
new rspack.container.ContainerPlugin({
  name: 'reusable-layered',
  enhanced: true,
  exposes: reusableLayeredExposes,
});
// @ts-expect-error A reusable enhanced map must not bypass the legacy gate.
export const legacyExposeMap: ExposesObject<false> = reusableLayeredExposes;
// @ts-expect-error A reusable enhanced map requires enhanced: true.
new rspack.container.ContainerPlugin({
  name: 'legacy-map',
  exposes: reusableLayeredExposes,
});
new rspack.container.ModuleFederationPluginV1({
  name: 'legacy-map',
  // @ts-expect-error V1 also requires the enhanced gate for a reusable layered map.
  exposes: reusableLayeredExposes,
});
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

const enhancedV1Options: EnhancedModuleFederationPluginV1Options = {
  name: 'enhanced-v1',
  enhanced: true,
  exposes: {
    './entry': { import: './index', layer: 'server' },
  },
};
new rspack.container.ModuleFederationPluginV1(enhancedV1Options);

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

// Consumer-only containers preserve main's optional-name contract.
new rspack.container.ModuleFederationPlugin({ shared: ['react'] });
const unnamedEnhancedConsumer: EnhancedModuleFederationPluginV1Options = {
  enhanced: true,
  shared: ['react'],
};
new rspack.container.ModuleFederationPluginV1(unnamedEnhancedConsumer);
