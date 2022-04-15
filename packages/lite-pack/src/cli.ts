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
  build({
    input: {
      main: path.resolve(argv.root, 'index.js')
    },
    root: argv.root
  })
}).help().argv;