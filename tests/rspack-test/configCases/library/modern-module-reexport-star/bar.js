import { useless } from '../modern-module-reexport-star/value'

(function() {
	// side effect access
	console.bind(useless)
})()
