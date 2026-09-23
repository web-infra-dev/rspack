/** @type {import("@rspack/core").Configuration} */
export default {
  optimization: {
    concatenateModules: true,
  },
  externals: [
    ({ context, request, getResolve }) => {
      if (request !== 'external') return false;
      const resolveFunction = getResolve();
      return new Promise((resolve, reject) => {
        resolveFunction(context, request, (err, resource, resolveData) => {
          if (err) {
            reject(err);
            return;
          }
          resolve(
            `var ${JSON.stringify({ resource, esm: resolveData.descriptionFileData.type === 'module' })}`,
          );
        });
      });
    },
  ],
};
