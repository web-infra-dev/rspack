import fs from 'fs'

import { join, resolve } from 'path'

it('should compile', () => {
	console.log.bind([fs, join, resolve])
})
