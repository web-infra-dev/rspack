module.exports = function(config) {
  console.log('Complex runtime plugin loaded with config:', config);

  const { nestedConfig, callbackName } = config;

  let initialized = false;

  const plugin = {
    name: 'complex-plugin',
    config: config,
    isInitialized: false,

    init: function(federation) {
      if (this.isInitialized) return;

      this.isInitialized = true;
      initialized = true;
      console.log('Complex plugin initialized with federation instance');

      if (nestedConfig && nestedConfig.enabled) {
        console.log('Nested config enabled with options:', nestedConfig.options);
      }

      if (typeof window !== 'undefined' && callbackName) {
        window[callbackName] = function() {
          return 'Callback from complex plugin';
        };
      }
    },

    getState: function() {
      return {
        initialized,
        config: this.config,
        nestedConfig: this.config.nestedConfig
      };
    }
  };

  return plugin;
};
