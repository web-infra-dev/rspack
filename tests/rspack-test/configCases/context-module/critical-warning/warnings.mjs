export default [
  /Dynamic module path: Rspack may bundle multiple modules that match this path pattern\./,
  /Dynamic module path: Rspack may bundle multiple modules that match this path pattern\./,
  [
    /Dynamic module path: Rspack cannot determine which modules to bundle because the path is unknown at build time/,
    /To bundle local modules, use a literal path or a pattern/,
    /To load the module at runtime,.*rspackIgnore: true/,
    /set `parser\.importDynamic` to `false` in a rule matching that package/,
  ],
];
