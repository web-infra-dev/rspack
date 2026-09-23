import { defineConfig } from '@rspack/cli';

export default defineConfig({
  optimization: {
    concatenateModules: true,
  },
  externals: [
    ({ context, request, getResolve }) => {
      if (request !== 'external') return false;
      const resolveFunction = getResolve!();
      return new Promise<string>((resolve, reject) => {
        resolveFunction(context!, request, (err, resource, resolveData) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(
            `var ${JSON.stringify({ resource, esm: resolveData?.descriptionFileData?.type === 'module' })}`,
          );
        });
      });
    },
  ],
});
