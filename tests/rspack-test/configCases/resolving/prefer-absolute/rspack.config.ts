import { defineConfig } from '@rspack/cli';

export default defineConfig({
  resolve: {
    preferAbsolute: true,
    plugins: [
      {
        apply(resolver: {
          hooks: {
            file: {
              tap(
                name: string,
                callback: (request: { path: string }) => void,
              ): void;
            };
          };
        }) {
          resolver.hooks.file.tap('Test', (request) => {
            if (/test.configCases.*test.configCases/.test(request.path))
              throw new Error('Trying to resolve as root path');
          });
        },
      },
    ],
  },
});
