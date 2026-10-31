import path from 'node:path';

export default function (code) {
  // Runs directly after builtin:lightningcss-loader, before Rspack parses the module.
  switch (path.basename(this.resourcePath)) {
    case 'component.css':
      // One undefined name leaves all custom media in the file unchanged.
      expect(code).toContain('@custom-media --narrow (width <= 500px);');
      expect(code).toContain('@media (--narrow) {\n  .narrow {');
      expect(code).toContain('@media (--wide) {\n  .wide {');
      break;
    case 'definitions.css':
      expect(code).not.toContain('@custom-media');
      break;
  }

  return code;
}
