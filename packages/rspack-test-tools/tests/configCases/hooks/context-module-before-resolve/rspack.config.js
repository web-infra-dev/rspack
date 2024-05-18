const pluginName = "plugin";

class Plugin {
    apply(compiler) {
        compiler.hooks.contextModuleFactory.tap(
            pluginName,
            contextModuleFactory => {
                contextModuleFactory.hooks.beforeResolve.tap(pluginName, resolveData => {
                    if (resolveData.request.includes("./locale")) {
                        resolveData.regExp = /[/\\](en(\.js)?|zh(\.js)?)$/;
                        return resolveData;
                    }
                });
            }
        );
    }
}
/**@type {import("@rspack/core").Configuration}*/
module.exports = {
    context: __dirname,
    entry: "./index.js",
    module: {
        rules: []
    },
    plugins: [new Plugin()]
};
