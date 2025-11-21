import { resolve } from 'path'
import { readFileSync } from 'fs'

const file = readFileSync(resolve(__dirname, 'module.js'), 'utf-8')

expect(file).toContain('import { resolve } from "path')
