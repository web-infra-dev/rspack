import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  optimization: {
    minimize: false,
    concatenateModules: false,
    sideEffects: true,
  },
  module: {
    rules: [
      {
        test: /pkg[/\\].*\.js$/,
        descriptionData: { 'component.scope': /^react$/ },
        use: ['./loader.cjs'],
      },
      {
        test: /pkg[/\\].*\.js$/,
        descriptionData: {
          component: (value) => {
            expect(value).toEqual({
              scope: 'react',
              nullable: null,
              list: ['a', 'b'],
            });
            return true;
          },
          'component.nullable': (value) => {
            expect(value).toBe(null);
            return true;
          },
          'component.list': (value) => {
            expect(value).toEqual(['a', 'b']);
            return true;
          },
          'component.missing': (value) => {
            expect(value).toBe('');
            return true;
          },
          // Property paths do not interpret array indices as object keys.
          'component.list.0': (value) => {
            expect(value).toBe('');
            return true;
          },
        },
      },
    ],
  },
});
