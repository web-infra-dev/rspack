import path from 'path'
import binding from '..'
import { RawOptions } from '../binding'

describe('binding', () => {
  it('work', async () => {
    const options: RawOptions = {
      entries: {
        main: {
          import: path.resolve(__dirname, './index.js')
        }
      },
      // entryFilename: path.resolve(__dirname, 'dist/main.js'),
    }
    const instance = binding.newRspack(JSON.stringify(options))
    await binding.build(instance)
    // setTimeout(() => {
    //   log();
    // }, 5000);
  })
})
