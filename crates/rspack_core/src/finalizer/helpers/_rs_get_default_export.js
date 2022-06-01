// get default export and interop with esm
rs._get_default_export = (module) => {
  var getter = module && module.__esModule ? () => module['default'] : () => module;
  return getter;
};
