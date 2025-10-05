it("should load runtime plugins correctly", () => {
  return import("./App").then(({ default: App }) => {
    const renderedResult = App();
    expect(renderedResult).toContain("Runtime plugins test component");

    expect(typeof __webpack_require__).toBe("function");
    expect(typeof __webpack_require__.federation).toBe("object");
    expect(typeof __webpack_require__.federation.initOptions).toBe("object");

    const plugins = __webpack_require__.federation.initOptions.plugins;
    expect(Array.isArray(plugins)).toBe(true);
    expect(plugins.length).toBeGreaterThanOrEqual(3);

    const hasBasicPlugin = plugins.some(plugin =>
      typeof plugin === "string" && plugin.includes("plugin.js")
    );
    expect(hasBasicPlugin).toBe(true);

    const paramPlugin = plugins.find(plugin =>
      Array.isArray(plugin) && plugin[0].includes("plugin-with-params.js")
    );
    expect(Array.isArray(paramPlugin)).toBe(true);
    expect(paramPlugin[1]).toBeDefined();
    expect(paramPlugin[1].testParam1).toBe("value1");
    expect(paramPlugin[1].testParam2).toBe(123);
    expect(paramPlugin[1].testParam3).toBe(true);

    const complexPlugin = plugins.find(plugin =>
      Array.isArray(plugin) && plugin[0].includes("complex-plugin.js")
    );
    expect(Array.isArray(complexPlugin)).toBe(true);
    expect(complexPlugin[1]).toBeDefined();
    expect(complexPlugin[1].nestedConfig).toBeDefined();
    expect(complexPlugin[1].nestedConfig.enabled).toBe(true);
    expect(Array.isArray(complexPlugin[1].nestedConfig.options)).toBe(true);
  });
});

it("should have correct runtime plugins structure", () => {
  return import("./container.js").then((container) => {
    expect(typeof container).toBe("object");
    expect(typeof container.get).toBe("function");
    expect(typeof container.init).toBe("function");

    expect(typeof __module_federation_runtime_plugins__).toBe("object");
    expect(Array.isArray(__module_federation_runtime_plugins__)).toBe(true);
  });
});
