import rspack, {
  type ConsumesConfig,
  type ConsumeSharedPluginOptions,
  type ContainerPluginOptions,
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

const annotatedEnhancedContainer: ContainerPluginOptions = {
  name: 'annotated-enhanced',
  enhanced: true,
  exposes: { './entry': { import: './index', layer: 'server' } },
};
new rspack.container.ContainerPlugin(annotatedEnhancedContainer);

// @ts-expect-error Expose layers require the enhanced runtime gate.
new rspack.container.ContainerPlugin({
  name: 'legacy',
  exposes: {
    './entry': { import: './index', layer: 'server' },
  },
});
