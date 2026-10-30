import rspack, {
  type ConsumesConfig,
  type ConsumeSharedPluginOptions,
  type EnhancedConsumeSharedPluginOptions,
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
