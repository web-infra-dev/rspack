import f1, { readFile } from 'fs'
import f2, { readFile as readFile2 } from './other'
import f3, { readFile3 } from './other2'

it('should render external correctly', async () => {
  const {ns} = await import('./async')

  expect(f1).toBe(f2).toBe(f3).toBe(ns.default)
  expect(readFile).toBe(readFile2).toBe(readFile3).toBe(ns.readFile)
})