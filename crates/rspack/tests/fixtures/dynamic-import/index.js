const request = 'c'
import('./a.js').then(({ a }) => console.log(a))
import(`./b.js`).then(({b}) => console.log(b))
import(`./${request}.js`).then(({c}) => console.log(c))