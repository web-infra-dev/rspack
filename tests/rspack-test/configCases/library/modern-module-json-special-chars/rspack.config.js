module.exports ={
entry: {
main: {import: './index.js', filename: 'bundle.mjs'},
json: {import: './data.json', filename: 'json.mjs'},
},
output: {
module: true,
library: {
type: 'modern-module',
},
},
module: {
parser: {
javascript: {
importMeta: false
}
}
},
}
