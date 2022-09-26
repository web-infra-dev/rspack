# github/three:
# 	mkdir -p github
# 	git clone --depth 1 --branch r108 https://github.com/mrdoob/three.js.git github/three

copy/three:
	mkdir -p benchcases/three/src
	echo > benchcases/three/src/entry.js
	for i in 1 2 3 4 5 6 7 8 9 10; do test -d "benchcases/three/src/copy$$i" || cp -r examples/.three/src "benchcases/three/src/copy$$i"; done
	for i in 1 2 3 4 5 6 7 8 9 10; do echo "import * as copy$$i from './copy$$i/Three.js'; export {copy$$i}" >> benchcases/three/src/entry.js; done
	echo "module.exports = {mode: 'development',entry: {index: './src/entry.js'}};" > benchcases/three/test.config.js
	echo "module.exports = {mode: 'development', entry: {index: './src/entry.js',devtool: 'eval',cache: {type: 'filesystem'}}}" > benchcases/three/webpack.config.js



bench_three: | copy/three
	@cargo build -p bench --release
	@echo "rspack"
	@hyperfine --warmup 3 ./target/release/bench
	@echo "esbuild"
	@hyperfine --warmup 3 './node_modules/esbuild/bin/esbuild --bundle --global-name=THREE  benchcases/three/src/entry.js --outfile=benchcases/three/esbuild/entry.esbuild.js --timing'
	@echo "webpack"
	@hyperfine --warmup 3 'npx webpack --entry ./benchcases/three/src/entry.js --mode development -c ./benchcases/three/webpack.config.js'
