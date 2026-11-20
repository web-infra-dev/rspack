export default [
  /Rspack may include multiple modules because part of this import path is dynamic\./,
  /Rspack may include multiple modules because part of this import path is dynamic\./,
  [
    /Rspack cannot determine which modules to bundle for this import/,
    /To bundle local modules, use a literal path or a pattern/,
    /To load the module at runtime,.*rspackIgnore: true/,
  ],
];
