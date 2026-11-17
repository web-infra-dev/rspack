// Keep the actual factory and module-local loader observable to webpack interceptors.
// The native child context also carries HMR state that does not live on the loader.
Object.defineProperties(__rspack_context, {
  __rspack_call_factory__: {
    value: function(options) {
      var module = options.module;
      if (options.factory === __webpack_modules__[options.id] && options.require === options.context.r) {
        return options.factory.call(module.exports, module, module.exports, options.context);
      }
      Object.defineProperty(module, "__rspack_runtime_context__", { configurable: true, value: options.context });
      return options.factory.call(module.exports, module, module.exports, options.require);
    }
  },
  __rspack_with_require__: {
    value: function(require) {
      if (require === this.r) return this;
      var context = Object.create(this);
      Object.defineProperty(context, "r", { configurable: true, value: require });
      Object.keys(require).forEach(function(key) {
        Object.defineProperty(context, key === "r" ? "N" : key, {
          configurable: true, enumerable: true,
          get: function() { return require[key]; },
          set: function(value) { require[key] = value; }
        });
      });
      return context;
    }
  }
});
