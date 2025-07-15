#!/bin/bash
# cspell:ignore threejs gltf draco rgbe jsm

echo "🔍 Three.js Module Federation Sharing Benchmark"
echo "=============================================="

cd examples/basic

# First, ensure we have three.js installed
echo "📦 Installing Three.js..."
pnpm add three@0.169.0 --workspace-root=false

# Create a Three.js test application
echo "📝 Creating Three.js test application..."

# Create the Three.js app structure
mkdir -p threejs-app/src

# Create main entry file that uses Three.js
cat > threejs-app/src/main.js << 'EOF'
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { RGBELoader } from 'three/examples/jsm/loaders/RGBELoader.js';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/examples/jsm/postprocessing/UnrealBloomPass.js';

console.log('Three.js version:', THREE.REVISION);

// Create a scene
export function createScene() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    
    renderer.setSize(window.innerWidth, window.innerHeight);
    
    // Add lights
    const ambientLight = new THREE.AmbientLight(0x404040);
    scene.add(ambientLight);
    
    const directionalLight = new THREE.DirectionalLight(0xffffff, 1);
    directionalLight.position.set(5, 5, 5);
    scene.add(directionalLight);
    
    // Add geometry
    const geometry = new THREE.BoxGeometry(1, 1, 1);
    const material = new THREE.MeshPhongMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);
    
    camera.position.z = 5;
    
    // Add controls
    const controls = new OrbitControls(camera, renderer.domElement);
    
    // Add post-processing
    const composer = new EffectComposer(renderer);
    const renderPass = new RenderPass(scene, camera);
    composer.addPass(renderPass);
    
    const bloomPass = new UnrealBloomPass(
        new THREE.Vector2(window.innerWidth, window.innerHeight),
        1.5, 0.4, 0.85
    );
    composer.addPass(bloomPass);
    
    return { scene, camera, renderer, cube, controls, composer };
}

// Export loaders for use in other modules
export { GLTFLoader, DRACOLoader, RGBELoader };
export { THREE };
EOF

# Create additional modules that also use Three.js
cat > threejs-app/src/materials.js << 'EOF'
import * as THREE from 'three';

export function createAdvancedMaterials() {
    return {
        standard: new THREE.MeshStandardMaterial({
            color: 0x2194ce,
            roughness: 0.5,
            metalness: 0.5
        }),
        physical: new THREE.MeshPhysicalMaterial({
            color: 0xff0000,
            roughness: 0.2,
            metalness: 0.8,
            clearcoat: 1.0,
            clearcoatRoughness: 0.1
        }),
        toon: new THREE.MeshToonMaterial({
            color: 0x00ff00
        }),
        lambert: new THREE.MeshLambertMaterial({
            color: 0x0000ff
        })
    };
}

export class MaterialFactory {
    constructor() {
        this.cache = new Map();
    }
    
    getMaterial(type, options) {
        const key = `${type}_${JSON.stringify(options)}`;
        if (!this.cache.has(key)) {
            this.cache.set(key, new THREE[type](options));
        }
        return this.cache.get(key);
    }
}
EOF

cat > threejs-app/src/geometries.js << 'EOF'
import * as THREE from 'three';

export function createGeometries() {
    return {
        box: new THREE.BoxGeometry(1, 1, 1),
        sphere: new THREE.SphereGeometry(1, 32, 32),
        cylinder: new THREE.CylinderGeometry(1, 1, 2, 32),
        torus: new THREE.TorusGeometry(1, 0.4, 16, 100),
        torusKnot: new THREE.TorusKnotGeometry(1, 0.3, 100, 16),
        plane: new THREE.PlaneGeometry(5, 5),
        cone: new THREE.ConeGeometry(1, 2, 32),
        dodecahedron: new THREE.DodecahedronGeometry(1),
        icosahedron: new THREE.IcosahedronGeometry(1),
        octahedron: new THREE.OctahedronGeometry(1),
        tetrahedron: new THREE.TetrahedronGeometry(1)
    };
}

export class GeometryProcessor {
    static merge(geometries) {
        const merged = new THREE.BufferGeometry();
        // Simplified merge logic
        return merged;
    }
    
    static optimize(geometry) {
        geometry.computeVertexNormals();
        geometry.computeBoundingBox();
        geometry.computeBoundingSphere();
        return geometry;
    }
}
EOF

# Create entry file
cat > threejs-app/index.js << 'EOF'
import { createScene, THREE, GLTFLoader, DRACOLoader } from './src/main.js';
import { createAdvancedMaterials, MaterialFactory } from './src/materials.js';
import { createGeometries, GeometryProcessor } from './src/geometries.js';

console.log('Starting Three.js application...');

// Initialize scene
const { scene, camera, renderer, cube } = createScene();

// Create materials
const materials = createAdvancedMaterials();
const materialFactory = new MaterialFactory();

// Create geometries
const geometries = createGeometries();

// Use various Three.js features
const group = new THREE.Group();
Object.entries(geometries).forEach(([name, geometry], index) => {
    const material = Object.values(materials)[index % Object.keys(materials).length];
    const mesh = new THREE.Mesh(geometry, material);
    mesh.position.x = (index % 4) * 3 - 4.5;
    mesh.position.y = Math.floor(index / 4) * 3 - 3;
    group.add(mesh);
});

scene.add(group);

// Test loader usage
const gltfLoader = new GLTFLoader();
const dracoLoader = new DRACOLoader();
gltfLoader.setDRACOLoader(dracoLoader);

console.log('Three.js app initialized with', Object.keys(geometries).length, 'geometries');

export { scene, camera, renderer };
EOF

# Create config with Three.js as shared module
cat > rspack.config.threejs-shared.cjs << 'EOF'
const rspack = require("@rspack/core");
const path = require("path");

module.exports = {
	context: __dirname,
	entry: {
		main: "./threejs-app/index.js"
	},
	mode: "development",
	devtool: false,
	output: {
		clean: true,
		path: path.join(__dirname, "dist-threejs-shared")
	},
	resolve: {
		extensions: ['.js', '.jsx', '.ts', '.tsx']
	},
	optimization: {
		minimize: false,
		usedExports: true,
		providedExports: true,
		sideEffects: true, // Three.js has side effects
		concatenateModules: false,
		innerGraph: true
	},
	plugins: [
		new rspack.container.ModuleFederationPlugin({
			name: "threejs_app",
			filename: "remoteEntry.js",
			
			exposes: {
				"./Scene": "./threejs-app/src/main.js",
				"./Materials": "./threejs-app/src/materials.js",
				"./Geometries": "./threejs-app/src/geometries.js"
			},
			
			shared: {
				// Share Three.js and all its modules
				"three": {
					singleton: true,
					eager: false,
					requiredVersion: "^0.169.0",
					shareKey: "three",
					shareScope: "default"
				},
				"three/examples/jsm/controls/OrbitControls.js": {
					singleton: true,
					eager: false,
					shareKey: "three-orbit-controls"
				},
				"three/examples/jsm/loaders/GLTFLoader.js": {
					singleton: true,
					eager: false,
					shareKey: "three-gltf-loader"
				},
				"three/examples/jsm/loaders/DRACOLoader.js": {
					singleton: true,
					eager: false,
					shareKey: "three-draco-loader"
				},
				"three/examples/jsm/loaders/RGBELoader.js": {
					singleton: true,
					eager: false,
					shareKey: "three-rgbe-loader"
				},
				"three/examples/jsm/postprocessing/EffectComposer.js": {
					singleton: true,
					eager: false,
					shareKey: "three-effect-composer"
				},
				"three/examples/jsm/postprocessing/RenderPass.js": {
					singleton: true,
					eager: false,
					shareKey: "three-render-pass"
				},
				"three/examples/jsm/postprocessing/UnrealBloomPass.js": {
					singleton: true,
					eager: false,
					shareKey: "three-bloom-pass"
				}
			}
		})
	]
};
EOF

# Create baseline config without sharing
cat > rspack.config.threejs-baseline.cjs << 'EOF'
const path = require("path");

module.exports = {
	context: __dirname,
	entry: {
		main: "./threejs-app/index.js"
	},
	mode: "development",
	devtool: false,
	output: {
		clean: true,
		path: path.join(__dirname, "dist-threejs-baseline")
	},
	resolve: {
		extensions: ['.js', '.jsx', '.ts', '.tsx']
	},
	optimization: {
		minimize: false,
		usedExports: true,
		providedExports: true,
		sideEffects: true,
		concatenateModules: false,
		innerGraph: true
	}
	// NO MODULE FEDERATION
};
EOF

echo ""
echo "🏃 Running Three.js sharing benchmark..."

# Function to run benchmark
run_benchmark() {
    local name=$1
    local config=$2
    local iterations=5
    local times=()
    
    echo "" >&2
    echo "🎯 $name ($iterations iterations)..." >&2
    
    for i in $(seq 1 $iterations); do
        rm -rf dist-threejs-*
        start=$(date +%s%N)
        node ../../packages/rspack-cli/bin/rspack.js build --config $config > /dev/null 2>&1
        end=$(date +%s%N)
        duration=$(( (end - start) / 1000000 ))
        times+=($duration)
        echo "  Run $i: ${duration}ms" >&2
    done
    
    # Calculate stats
    sum=0
    min=${times[0]}
    max=${times[0]}
    for time in "${times[@]}"; do
        sum=$((sum + time))
        if [ $time -lt $min ]; then min=$time; fi
        if [ $time -gt $max ]; then max=$time; fi
    done
    avg=$((sum / iterations))
    
    echo "  Average: ${avg}ms (min: ${min}ms, max: ${max}ms)" >&2
    echo $avg
}

# Run benchmarks
baseline_time=$(run_benchmark "Baseline (No Module Federation)" "rspack.config.threejs-baseline.cjs")
shared_time=$(run_benchmark "Three.js as Shared Module" "rspack.config.threejs-shared.cjs")

# Calculate impact
impact=$(( (shared_time - baseline_time) * 100 / baseline_time ))

echo ""
echo "📊 Three.js Module Federation Performance Analysis:"
echo "=================================================="
echo "  Baseline (no sharing):         ${baseline_time}ms"
echo "  Three.js as shared module:     ${shared_time}ms (+${impact}%)"
echo ""

if [ $impact -gt 30 ]; then
    echo "🚨 SIGNIFICANT PERFORMANCE REGRESSION: +${impact}%"
    echo "   Sharing Three.js has a major performance impact!"
elif [ $impact -gt 15 ]; then
    echo "⚠️  Notable performance impact: +${impact}%"
else
    echo "✅ Acceptable performance impact: +${impact}%"
fi

# Get bundle size comparison
echo ""
echo "📦 Bundle Size Analysis:"
if [ -d "dist-threejs-baseline" ] && [ -d "dist-threejs-shared" ]; then
    baseline_size=$(du -sk dist-threejs-baseline | cut -f1)
    shared_size=$(du -sk dist-threejs-shared | cut -f1)
    size_diff=$(( (shared_size - baseline_size) * 100 / baseline_size ))
    
    echo "  Baseline bundle:    ${baseline_size}KB"
    echo "  Shared bundle:      ${shared_size}KB (${size_diff}% difference)"
    
    # Count files
    baseline_files=$(find dist-threejs-baseline -name "*.js" | wc -l)
    shared_files=$(find dist-threejs-shared -name "*.js" | wc -l)
    echo "  Baseline files:     ${baseline_files}"
    echo "  Shared files:       ${shared_files}"
fi

# Get detailed stats
echo ""
echo "📈 Module Federation Statistics:"
node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.threejs-shared.cjs --json > threejs-stats.json 2>/dev/null

if [ -f "threejs-stats.json" ]; then
    modules=$(jq -r '.modules | length // "N/A"' threejs-stats.json)
    chunks=$(jq -r '.chunks | length // "N/A"' threejs-stats.json)
    assets=$(jq -r '.assets | length // "N/A"' threejs-stats.json)
    
    echo "  Total modules:      $modules"
    echo "  Total chunks:       $chunks"  
    echo "  Total assets:       $assets"
    
    # Show shared modules info
    echo ""
    echo "  Shared modules detected:"
    jq -r '.modules[]? | select(.providedExports) | select(.identifier | contains("three")) | .identifier' threejs-stats.json 2>/dev/null | head -10 | sed 's/^/    - /'
fi

# Clean up
rm -f rspack.config.threejs-*.cjs threejs-stats.json
rm -rf threejs-app

cd ../..

echo ""
echo "✅ Three.js sharing benchmark complete!"
echo ""
echo "💡 This benchmark demonstrates the performance impact of sharing"
echo "   a large, real-world library (Three.js) through Module Federation."
echo ""
echo "🎯 Key insights:"
echo "   - Three.js has many internal modules and dependencies"
echo "   - Sharing it requires analyzing all its imports/exports"
echo "   - The is_consume_shared_descendant check runs for each module"
echo "   - Performance impact shows the need for optimization"