import fileURLToPathAsset from './fileURLToPath.mjs'
import readFileAsset from './readFile.mjs'
import { readFile } from 'virtual-fs'
import { fileURLToPath } from 'virtual-url'

new URL('./fileURLToPath.mjs', import.meta.url)
new URL('./readFile.mjs', import.meta.url)

const generatedDirname = __dirname

it('should deconflict asset imports from external module bindings', () => {
  expect(fileURLToPathAsset).toBe('file URL asset')
  expect(readFileAsset).toBe('read file asset')
  expect(typeof readFile).toBe('function')
  expect(typeof fileURLToPath).toBe('function')
  expect(typeof generatedDirname).toBe('string')
})

export {
  fileURLToPath,
  fileURLToPathAsset,
  generatedDirname,
  readFile,
  readFileAsset,
}
