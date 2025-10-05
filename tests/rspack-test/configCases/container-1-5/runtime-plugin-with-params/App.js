export default () => {
  const hasRuntimePlugins = typeof __webpack_require__ !== 'undefined' &&
    typeof __webpack_require__.federation !== 'undefined' &&
    typeof __webpack_require__.federation.initOptions !== 'undefined' &&
    Array.isArray(__webpack_require__.federation.initOptions.plugins);

  const pluginsCount = hasRuntimePlugins ?
    __webpack_require__.federation.initOptions.plugins.length : 0;

  const hasBasicPlugin = hasRuntimePlugins &&
    __webpack_require__.federation.initOptions.plugins.some(plugin =>
      plugin.includes('plugin.js')
    );

  const hasParamPlugin = hasRuntimePlugins &&
    __webpack_require__.federation.initOptions.plugins.some(plugin =>
      Array.isArray(plugin) && plugin[0].includes('plugin-with-params.js')
    );

  // 返回状态信息
  return `Runtime plugins test component: [pluginsCount=${pluginsCount}, hasBasicPlugin=${hasBasicPlugin}, hasParamPlugin=${hasParamPlugin}]`;
};
