const { build, rspack } = require('@rspack/core');
const path = require('path');
const fs = require('fs');
async function main() {
  const configPath = path.resolve(module.parent.path, 'webpack.config.js');
  const defaultEntry = {
    entry: {
      main: './example.js',
    },
    context: module.parent.path,
  };
  let config = {};
  if (fs.existsSync(configPath)) {
    config = require(configPath);
  }
  try{
    await rspack({ ...defaultEntry, ...config });
  }catch(err){
    console.log(`build ${module.parent.path} failed:`,err);
    process.exit(1);
  }
  
}
main();
