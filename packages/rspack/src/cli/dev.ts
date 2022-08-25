import { Rspack } from '..';
import { CommonCLIOptions, CommonCommand } from './common';
import { loadConfigFile } from '../loadConfig';
import { createServer } from '../server';
import path from 'path';
interface DevCLIOptions extends CommonCLIOptions {
  open?: boolean;
  port?: number
}

class DevCommand extends CommonCommand {
  async execute(): Promise<number | void> {
    await devExecutor(this);
  }
}

async function devExecutor(options: DevCLIOptions) {
  const root = process.cwd();
  const userConfig = loadConfigFile(root, options.config);
  const rspack = new Rspack(userConfig);

  const buildStats = await rspack.build();
  console.log('buildStats', buildStats);

  const server = createServer(rspack.options)
  // TODO: should optimized
  server.watcher.on("change", async (filePath, stats) => {
    console.log('changed file:', filePath);
    const diff = await rspack.rebuild([filePath]);

    let relativePath = path.relative(rspack.options.context || process.cwd(), filePath);
    if (!(relativePath.startsWith('../') || relativePath.startsWith('./'))) {
      relativePath = './' + relativePath;
    }

    let code = `__rspack_runtime__.installedModules[${JSON.stringify(relativePath)}] = ${Object.values(diff)}; __rspack_runtime__.invalidate(${JSON.stringify(relativePath)})`
    server.ws.broad({
      type: 'js-hmr',
      path: relativePath,
      timestamp: Date.now(),
      code,
    })
    console.log('rebuildStats', diff)
  })
  await server.start();
}

export default DevCommand;
