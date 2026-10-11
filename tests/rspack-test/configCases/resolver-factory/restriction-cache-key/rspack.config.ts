import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import path from 'node:path';

const options = (restrictions: (string | RegExp)[]) => ({
  extensions: ['.css'],
  mainFields: ['main'],
  restrictions,
});

class Plugin {
  apply(compiler: Compiler) {
    const context = import.meta.dirname;
    const css = path.join(context, 'node_modules/style/index.css');
    const js = path.join(context, 'node_modules/style/index.js');

    {
      const cssResolver = compiler.resolverFactory.get(
        'normal',
        options([/\.css$/]),
      );
      const jsResolver = compiler.resolverFactory.get(
        'normal',
        options([/\.js$/]),
      );
      expect(cssResolver).not.toBe(jsResolver);
      expect(cssResolver.resolveSync({}, context, 'style')).toBe(css);
      expect(jsResolver.resolveSync({}, context, 'style')).toBe(js);
    }

    {
      const caseSensitive = compiler.resolverFactory.get(
        'normal',
        options([/\.CSS$/]),
      );
      const caseInsensitive = compiler.resolverFactory.get(
        'normal',
        options([/\.CSS$/i]),
      );
      expect(caseSensitive).not.toBe(caseInsensitive);
      expect(caseInsensitive.resolveSync({}, context, 'style')).toBe(css);
      expect(() => caseSensitive.resolveSync({}, context, 'style')).toThrow();
    }

    {
      const base = compiler.resolverFactory.get('normal');
      const cssResolver = base.withOptions(options([/\.css$/]));
      const jsResolver = base.withOptions(options([/\.js$/]));
      expect(cssResolver).not.toBe(jsResolver);
      expect(cssResolver.resolveSync({}, context, 'style')).toBe(css);
      expect(jsResolver.resolveSync({}, context, 'style')).toBe(js);
    }

    {
      const first = compiler.resolverFactory.get('normal', options([/\.css$/]));
      const second = compiler.resolverFactory.get(
        'normal',
        options([/\.css$/]),
      );
      expect(first).toBe(second);
    }
  }
}

export default defineConfig({
  entry: './index.js',
  plugins: [new Plugin()],
});
