# github/three:
# 	mkdir -p github
# 	git clone --depth 1 --branch r108 https://github.com/mrdoob/three.js.git github/three
copy/three:

	mkdir -p benchcases/three/src
	echo > benchcases/three/src/entry.js
	for i in 1 2 3 4 5 6 7 8 9 10; do test -d "benchcases/three/src/copy$$i" || cp -r examples/.three/src "benchcases/three/src/copy$$i"; done
	for i in 1 2 3 4 5 6 7 8 9 10; do echo "import * as copy$$i from './copy$$i/Three.js'; export {copy$$i}" >> benchcases/three/src/entry.js; done
	echo "module.exports = {mode: 'development',entry: {index: {import: ['./src/entry.js']}}};" > benchcases/three/test.config.js
	echo "module.exports = {mode: 'development',entry: {index: ['./benchcases/three/src/entry.js']},devtool: 'eval',cache: {type: 'filesystem'}}" > benchcases/three/webpack.config.js

flamegraph:
	samply record ./target/release/bench

bench_three: | copy/three
	@cargo build -p bench --release
	@echo "rspack"
	@hyperfine --warmup 3  \
		-n rspack './target/release/bench' \
		-n esbuild  './node_modules/esbuild/bin/esbuild --bundle --global-name=THREE  benchcases/three/src/entry.js --outfile=benchcases/three/esbuild/entry.esbuild.js --timing' \
		-n webpack './node_modules/webpack-cli/bin/cli.js  -c ./benchcases/three/webpack.config.js'


esbuild_trace:
	./node_modules/esbuild/bin/esbuild --bundle benchcases/three/src/entry.js --outfile=/dev/null --trace=esbuild.trace
	go tool trace esbuild.trace

esbuild_cpuprofile:
	./node_modules/esbuild/bin/esbuild --bundle benchcases/three/src/entry.js --outfile=/dev/null --cpuprofile=esbuild.cpuprofile
	go tool pprof -http=:1234 esbuild.cpuprofile

rspack_trace:
	TRACE=TRACE cargo run -F tracing --release --bin bench


sync_bnpm:
	@bnpm sync @rspack/core
	@bnpm sync @rspack/dev-server
	@bnpm sync @rspack/dev-client
	@bnpm sync @rspack/dev-middleware
	@bnpm sync @rspack/plugin-less
	@bnpm sync @rspack/plugin-postcss
	@bnpm sync @rspack/cli
	@bnpm sync @rspack/binding
	@bnpm sync create-rspack
snapshot_ci:
	@pnpm version:snapshot
	@./x build js-release-all
	@pnpm release:snapshot
release-snapshot:
	@make snapshot_ci
	@make sync_bnpm