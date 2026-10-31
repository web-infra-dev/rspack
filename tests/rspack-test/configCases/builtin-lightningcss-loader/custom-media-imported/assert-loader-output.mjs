import path from 'node:path';

export default function (code) {
  // Runs directly after builtin:lightningcss-loader, before Rspack parses the module.
  expect(code).not.toContain('@custom-media');

  switch (path.basename(this.resourcePath)) {
    case 'component.css':
      expect(code).toContain('@media (width >= 800px) {\n  .component {');
      expect(code).toContain('@media (width <= 500px) {\n  .compact {');
      break;
    case 'override.css':
      expect(code).toContain('@media (width >= 1000px) {\n  .override {');
      break;
  }

  return code;
}
