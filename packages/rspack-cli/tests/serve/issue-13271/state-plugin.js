function createListeningLogger(compiler, label) {
  return (server) => {
    console.log(
      JSON.stringify({
        compiler: compiler.options.name,
        source: label,
        activeCompiler:
          server?.compiler?.options?.name ?? compiler.options.name,
      }),
    );
  };
}

module.exports = function statePlugin(label) {
  return {
    name: `issue-13271-${label}`,
    apply(compiler) {
      const devServer = compiler.options.devServer;
      if (!devServer || devServer === false) {
        return;
      }

      const previousOnListening = devServer.onListening;
      devServer.onListening = (server) => {
        createListeningLogger(compiler, label)(server);
        previousOnListening?.(server);
      };
    },
  };
};
