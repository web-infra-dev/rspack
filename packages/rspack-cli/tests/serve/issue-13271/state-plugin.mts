import type { Compiler } from '@rspack/core';
import type { RspackDevServer } from '@rspack/dev-server';

function createListeningLogger(compiler: Compiler, label: string) {
  return (server: RspackDevServer) => {
    const options = server?.compiler?.options;
    console.log(
      JSON.stringify({
        compiler: compiler.options.name,
        source: label,
        activeCompiler:
          (Array.isArray(options) ? undefined : options?.name) ??
          compiler.options.name,
      }),
    );
  };
}

export default function statePlugin(label: string) {
  return {
    name: `issue-13271-${label}`,
    apply(compiler: Compiler) {
      const devServer = compiler.options.devServer;
      if (!devServer) {
        return;
      }

      const previousOnListening = devServer.onListening;
      devServer.onListening = (server) => {
        createListeningLogger(compiler, label)(server);
        previousOnListening?.(server);
      };
    },
  };
}
