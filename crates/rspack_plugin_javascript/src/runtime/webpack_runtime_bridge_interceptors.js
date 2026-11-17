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
      // The wrapper can define private fields after calling the factory. Resolve them
      // against its actual loader when the module reads or writes them.
      if (typeof Proxy === "function") return new Proxy(context, {
        get: function(target, key, receiver) {
          var prop = key === "N" ? "r" : key;
          return key !== "r" && prop in require ? require[prop] : Reflect.get(target, key, receiver);
        },
        set: function(target, key, value, receiver) {
          var prop = key === "N" ? "r" : key;
          return key !== "r" && (prop in require || !(key in target)) ? Reflect.set(require, prop, value) : Reflect.set(target, key, value, receiver);
        },
        has: function(target, key) { return (key === "N" ? "r" : key) in require || key in target; }
      });
      var keys = Object.getOwnPropertyNames(require);
      if (Object.getOwnPropertySymbols) keys = keys.concat(Object.getOwnPropertySymbols(require));
      keys.forEach(function(key) {
        Object.defineProperty(context, key === "r" ? "N" : key, {
          configurable: true, enumerable: Object.prototype.propertyIsEnumerable.call(require, key),
          get: function() { return require[key]; },
          set: function(value) { require[key] = value; }
        });
      });
      return context;
    }
  }
});
