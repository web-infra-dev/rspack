var path = require( 'path' );
var fs = require( 'fs' );

// Creates a rollup config object for the given file to
// be converted to umd
function createOutput( file ) {

	var inputPath = path.resolve( file );
	var outputPath = inputPath.replace( /[\\\/]examples[\\\/]jsm[\\\/]/, '/examples/js/' );

	// Every import is marked as external so the output is 1-to-1. We
	// assume that that global object should be the THREE object so we
	// replicate the existing behavior.
	return {

		input: inputPath,
		treeshake: false,
		external: p => p !== inputPath,

		plugins: [ {

			generateBundle: function ( options, bundle ) {

				for ( var key in bundle ) {

					bundle[ key ].code = bundle[ key ].code.replace( /three_module_js/g, 'THREE' );

				}

			}

		} ],

		output: {

			format: 'umd',
			name: 'THREE',
			file: outputPath,

			globals: () => 'THREE',
			paths: p => /three\.module\.js$/.test( p ) ? 'three' : p,
			extend: true,

			banner:
				'/**\n' +
				` * Generated from '${ path.relative( '.', inputPath ).replace( /\\/g, '/' ) }'\n` +
				' */\n',
			esModule: false

		}

	};

}

// Walk the file structure starting at the given directory and fire
// the callback for every js file.
function walk( dir, cb ) {

	var files = fs.readdirSync( dir );
	files.forEach( f => {

		var p = path.join( dir, f );
		var stats = fs.statSync( p );

		if ( stats.isDirectory() ) {

			walk( p, cb );

		} else if ( f.endsWith( '.js' ) ) {

			cb( p );

		}

	} );

}

// Gather up all the files
var files = [];
walk( 'examples/jsm/', p => files.push( p ) );

// Create a rollup config for each module.js file
export default files.map( p => createOutput( p ) );
