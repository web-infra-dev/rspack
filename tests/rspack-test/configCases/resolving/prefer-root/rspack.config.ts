import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolve: {
    // @ts-expect-error Rspack does not support resolver plugins yet.
    plugins: [
      {
        apply(resolver: {
          hooks: {
            file: {
              tap(
                name: string,
                callback: (request: { path: string | false }) => void,
              ): void;
            };
          };
        }) {
          resolver.hooks.file.tap('Test', (request) => {
            if (request.path === '/index.js')
              throw new Error('Trying to resolve as absolute path');
          });
        },
      },
    ],
  },
});
