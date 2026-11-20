export default [
  /Rspack may include multiple modules because part of the module path is dynamic\./,
  /Rspack may include multiple modules because part of the module path is dynamic\./,
  [
    /Rspack cannot determine which modules to bundle because the module path is not known at build time/,
    /To bundle local modules, use a literal path or a pattern/,
    /To load the module at runtime,.*rspackIgnore: true/,
  ],
];
