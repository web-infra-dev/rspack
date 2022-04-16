import path from 'path';
const yargs: typeof import('yargs') = require('yargs');
import { build } from './compiler';

// build({ main: path.resolve(__dirname, '../fixtures/index.js') });

yargs
.scriptName("lite-pack")
.usage("$0 <root>")
.command("$0 [root]", 'start dev server', (yargs) => {
  yargs.positional('root', {
    type: 'string',
    default: process.cwd(),
    describe: 'project root'
  })
},(argv:any) => {
  console.log('argv:',argv)
  const pakgPath = path.resolve(argv.root, 'package.json');
  const pkg = require(pakgPath);
  let entry = pkg?.rspack?.entry;
  let manualChunk = pkg?.rspack?.manualChunks;
  if(!entry)  {
    entry = {
      main: path.resolve(argv.root, 'index.js')
    }
  }
  for(const [key,value] of Object.entries(entry)){
    entry[key] = path.resolve(argv.root, value as string);
  }
  console.log('entry:',entry, pkg,manualChunk);
  build({
    entry: entry,
    root: argv.root,
    manualChunks: manualChunk ?? {},
  })
}).help().argv;