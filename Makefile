# github/three:
# 	mkdir -p github
# 	git clone --depth 1 --branch r108 https://github.com/mrdoob/three.js.git github/three

copy/three:
	mkdir -p benchcases/three/src
	echo > benchcases/three/src/entry.js
	for i in 1 2 3 4 5 6 7 8 9 10; do test -d "benchcases/three/src/copy$$i" || cp -r examples/three/src "benchcases/three/src/copy$$i"; done
	for i in 1 2 3 4 5 6 7 8 9 10; do echo "import * as copy$$i from './copy$$i/Three.js'; export {copy$$i}" >> benchcases/three/src/entry.js; done