#!/bin/bash
# cspell:ignore threejs gltf draco rgbe jsm metalness clearcoat roughness

echo "🔍 Three.js Multi-App Module Federation Benchmark"
echo "==============================================="
echo "This benchmark simulates multiple apps sharing Three.js modules"
echo ""

cd examples/basic

# Ensure Three.js is installed
if [ ! -d "node_modules/three" ]; then
    echo "📦 Installing Three.js..."
    pnpm add three@0.169.0 --workspace-root=false
fi

# Create multiple apps that all use Three.js
echo "📝 Creating multiple Three.js applications..."

# App 1: Main visualization app
mkdir -p app1-viz/src
cat > app1-viz/src/index.js << 'EOF'
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js';
import { createSceneUtils } from '../../shared-three/scene-utils.js';
import { MaterialLibrary } from '../../shared-three/materials.js';

export function createVisualization() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    
    const utils = createSceneUtils(THREE);
    const materials = new MaterialLibrary(THREE);
    
    // Create complex scene
    for (let i = 0; i < 20; i++) {
        const geometry = utils.randomGeometry();
        const material = materials.getRandom();
        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.random().multiplyScalar(10);
        scene.add(mesh);
    }
    
    return { scene, camera, renderer };
}

console.log('App1: Visualization loaded');
EOF

# App 2: Editor app
mkdir -p app2-editor/src
cat > app2-editor/src/index.js << 'EOF'
import * as THREE from 'three';
import { TransformControls } from 'three/examples/jsm/controls/TransformControls.js';
import { BoxHelper } from 'three/src/helpers/BoxHelper.js';
import { GridHelper } from 'three/src/helpers/GridHelper.js';
import { createSceneUtils } from '../../shared-three/scene-utils.js';
import { GeometryFactory } from '../../shared-three/geometries.js';

export class Editor {
    constructor() {
        this.scene = new THREE.Scene();
        this.camera = new THREE.PerspectiveCamera(50, window.innerWidth / window.innerHeight, 0.1, 1000);
        this.renderer = new THREE.WebGLRenderer();
        this.utils = createSceneUtils(THREE);
        this.geometryFactory = new GeometryFactory(THREE);
        
        // Add editor helpers
        const grid = new GridHelper(20, 20);
        this.scene.add(grid);
        
        this.transformControls = new TransformControls(this.camera, this.renderer.domElement);
        this.scene.add(this.transformControls);
    }
    
    addPrimitive(type) {
        const geometry = this.geometryFactory.create(type);
        const material = new THREE.MeshStandardMaterial();
        const mesh = new THREE.Mesh(geometry, material);
        this.scene.add(mesh);
        return mesh;
    }
}

console.log('App2: Editor loaded');
EOF

# App 3: Game app
mkdir -p app3-game/src
cat > app3-game/src/index.js << 'EOF'
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/examples/jsm/loaders/DRACOLoader.js';
import { AnimationMixer } from 'three/src/animation/AnimationMixer.js';
import { createSceneUtils } from '../../shared-three/scene-utils.js';
import { PhysicsWorld } from '../../shared-three/physics.js';

export class GameEngine {
    constructor() {
        this.scene = new THREE.Scene();
        this.camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.1, 1000);
        this.renderer = new THREE.WebGLRenderer();
        this.clock = new THREE.Clock();
        this.mixers = [];
        
        this.utils = createSceneUtils(THREE);
        this.physics = new PhysicsWorld(THREE);
        
        // Setup loaders
        this.gltfLoader = new GLTFLoader();
        this.dracoLoader = new DRACOLoader();
        this.gltfLoader.setDRACOLoader(this.dracoLoader);
    }
    
    loadModel(url) {
        return this.gltfLoader.loadAsync(url).then(gltf => {
            this.scene.add(gltf.scene);
            if (gltf.animations.length > 0) {
                const mixer = new AnimationMixer(gltf.scene);
                this.mixers.push(mixer);
            }
            return gltf;
        });
    }
    
    update() {
        const delta = this.clock.getDelta();
        this.mixers.forEach(mixer => mixer.update(delta));
        this.physics.update(delta);
    }
}

console.log('App3: Game engine loaded');
EOF

# Create shared Three.js utilities
mkdir -p shared-three
cat > shared-three/scene-utils.js << 'EOF'
export function createSceneUtils(THREE) {
    return {
        randomGeometry() {
            const geometries = [
                new THREE.BoxGeometry(1, 1, 1),
                new THREE.SphereGeometry(0.5, 32, 16),
                new THREE.CylinderGeometry(0.5, 0.5, 1, 32),
                new THREE.TorusGeometry(0.5, 0.2, 16, 32),
                new THREE.ConeGeometry(0.5, 1, 32)
            ];
            return geometries[Math.floor(Math.random() * geometries.length)];
        },
        
        createLighting() {
            const lights = [];
            lights.push(new THREE.AmbientLight(0x404040));
            lights.push(new THREE.DirectionalLight(0xffffff, 1));
            lights.push(new THREE.PointLight(0xff0000, 1, 100));
            return lights;
        }
    };
}
EOF

cat > shared-three/materials.js << 'EOF'
export class MaterialLibrary {
    constructor(THREE) {
        this.THREE = THREE;
        this.materials = this.createMaterials();
    }
    
    createMaterials() {
        return [
            new this.THREE.MeshBasicMaterial({ color: 0xff0000 }),
            new this.THREE.MeshStandardMaterial({ color: 0x00ff00, roughness: 0.5 }),
            new this.THREE.MeshPhongMaterial({ color: 0x0000ff }),
            new this.THREE.MeshPhysicalMaterial({ color: 0xffffff, metallic: 1, roughness: 0 }),
            new this.THREE.MeshToonMaterial({ color: 0xff00ff })
        ];
    }
    
    getRandom() {
        return this.materials[Math.floor(Math.random() * this.materials.length)];
    }
}
EOF

cat > shared-three/geometries.js << 'EOF'
export class GeometryFactory {
    constructor(THREE) {
        this.THREE = THREE;
    }
    
    create(type) {
        switch(type) {
            case 'box': return new this.THREE.BoxGeometry(1, 1, 1);
            case 'sphere': return new this.THREE.SphereGeometry(0.5, 32, 16);
            case 'cylinder': return new this.THREE.CylinderGeometry(0.5, 0.5, 1);
            case 'torus': return new this.THREE.TorusGeometry(0.5, 0.2, 16, 32);
            case 'knot': return new this.THREE.TorusKnotGeometry(0.5, 0.15, 100, 16);
            default: return new this.THREE.BoxGeometry(1, 1, 1);
        }
    }
}
EOF

cat > shared-three/physics.js << 'EOF'
export class PhysicsWorld {
    constructor(THREE) {
        this.THREE = THREE;
        this.bodies = [];
        this.gravity = new THREE.Vector3(0, -9.8, 0);
    }
    
    addBody(mesh, mass = 1) {
        this.bodies.push({
            mesh,
            mass,
            velocity: new this.THREE.Vector3()
        });
    }
    
    update(delta) {
        this.bodies.forEach(body => {
            body.velocity.add(this.gravity.clone().multiplyScalar(delta));
            body.mesh.position.add(body.velocity.clone().multiplyScalar(delta));
        });
    }
}
EOF

# Create main entry that imports all apps
cat > multi-app-entry.js << 'EOF'
// Import all three apps
import { createVisualization } from './app1-viz/src/index.js';
import { Editor } from './app2-editor/src/index.js';
import { GameEngine } from './app3-game/src/index.js';

// Import shared utilities
import { createSceneUtils } from './shared-three/scene-utils.js';
import { MaterialLibrary } from './shared-three/materials.js';
import { GeometryFactory } from './shared-three/geometries.js';
import { PhysicsWorld } from './shared-three/physics.js';

// Initialize all apps
console.log('Initializing multi-app Three.js setup...');

const viz = createVisualization();
const editor = new Editor();
const game = new GameEngine();

// Test cross-app sharing
import * as THREE from 'three';
const sharedScene = new THREE.Scene();
const sharedUtils = createSceneUtils(THREE);

console.log('All apps initialized with shared Three.js modules');

export { viz, editor, game };
EOF

# Create config with extensive Three.js sharing
cat > rspack.config.multi-app-shared.cjs << 'EOF'
const rspack = require("@rspack/core");
const path = require("path");

module.exports = {
	context: __dirname,
	entry: {
		main: "./multi-app-entry.js"
	},
	mode: "development",
	devtool: false,
	output: {
		clean: true,
		path: path.join(__dirname, "dist-multi-shared")
	},
	optimization: {
		minimize: false,
		usedExports: true,
		providedExports: true,
		sideEffects: true,
		concatenateModules: false,
		innerGraph: true
	},
	plugins: [
		new rspack.container.ModuleFederationPlugin({
			name: "multi_app_three",
			filename: "remoteEntry.js",
			
			exposes: {
				"./App1": "./app1-viz/src/index.js",
				"./App2": "./app2-editor/src/index.js",
				"./App3": "./app3-game/src/index.js",
				"./SharedUtils": "./shared-three/scene-utils.js",
				"./SharedMaterials": "./shared-three/materials.js"
			},
			
			shared: {
				// Core Three.js
				"three": {
					singleton: true,
					eager: false,
					requiredVersion: "^0.169.0"
				},
				// Three.js modules - simulate heavy sharing
				"three/src/animation/AnimationMixer.js": { singleton: true },
				"three/src/helpers/BoxHelper.js": { singleton: true },
				"three/src/helpers/GridHelper.js": { singleton: true },
				"three/examples/jsm/controls/OrbitControls.js": { singleton: true },
				"three/examples/jsm/controls/TransformControls.js": { singleton: true },
				"three/examples/jsm/loaders/GLTFLoader.js": { singleton: true },
				"three/examples/jsm/loaders/DRACOLoader.js": { singleton: true },
				"three/examples/jsm/postprocessing/EffectComposer.js": { singleton: true },
				"three/examples/jsm/postprocessing/RenderPass.js": { singleton: true },
				
				// Shared utilities (these will trigger is_consume_shared_descendant checks)
				"./shared-three/scene-utils.js": {
					singleton: true,
					eager: false,
					shareKey: "scene-utils"
				},
				"./shared-three/materials.js": {
					singleton: true,
					eager: false,
					shareKey: "materials"
				},
				"./shared-three/geometries.js": {
					singleton: true,
					eager: false,
					shareKey: "geometries"
				},
				"./shared-three/physics.js": {
					singleton: true,
					eager: false,
					shareKey: "physics"
				}
			}
		})
	]
};
EOF

# Create baseline without sharing
cat > rspack.config.multi-app-baseline.cjs << 'EOF'
const path = require("path");

module.exports = {
	context: __dirname,
	entry: {
		main: "./multi-app-entry.js"
	},
	mode: "development",
	devtool: false,
	output: {
		clean: true,
		path: path.join(__dirname, "dist-multi-baseline")
	},
	optimization: {
		minimize: false,
		usedExports: true,
		providedExports: true,
		sideEffects: true,
		concatenateModules: false,
		innerGraph: true
	}
};
EOF

echo ""
echo "🏃 Running multi-app Three.js sharing benchmark..."

# Function to run benchmark
run_benchmark() {
    local name=$1
    local config=$2
    local iterations=5
    local times=()
    
    echo "" >&2
    echo "🎯 $name ($iterations iterations)..." >&2
    
    for i in $(seq 1 $iterations); do
        rm -rf dist-multi-*
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
baseline_time=$(run_benchmark "Baseline (No Module Federation)" "rspack.config.multi-app-baseline.cjs")
shared_time=$(run_benchmark "Multi-App Three.js Sharing" "rspack.config.multi-app-shared.cjs")

# Calculate impact
impact=$(( (shared_time - baseline_time) * 100 / baseline_time ))

echo ""
echo "📊 Multi-App Three.js Module Federation Analysis:"
echo "==============================================="
echo "  Baseline (no sharing):      ${baseline_time}ms"
echo "  Multi-app sharing:          ${shared_time}ms (+${impact}%)"
echo ""

if [ $impact -gt 40 ]; then
    echo "🚨 SEVERE PERFORMANCE REGRESSION: +${impact}%"
    echo "   Multiple apps sharing Three.js modules causes significant overhead!"
elif [ $impact -gt 20 ]; then
    echo "⚠️  Significant performance impact: +${impact}%"
elif [ $impact -gt 10 ]; then
    echo "⚠️  Notable performance impact: +${impact}%"
else
    echo "✅ Acceptable performance impact: +${impact}%"
fi

# Analyze sharing patterns
echo ""
echo "📈 Sharing Analysis:"
node ../../packages/rspack-cli/bin/rspack.js build --config rspack.config.multi-app-shared.cjs --json > multi-app-stats.json 2>/dev/null

if [ -f "multi-app-stats.json" ]; then
    total_modules=$(jq -r '.modules | length // 0' multi-app-stats.json)
    shared_count=$(jq -r '[.modules[]? | select(.providedExports) | select(.identifier | contains("shared-three"))] | length' multi-app-stats.json 2>/dev/null || echo "0")
    three_modules=$(jq -r '[.modules[]? | select(.identifier | contains("three"))] | length' multi-app-stats.json 2>/dev/null || echo "0")
    
    echo "  Total modules:              $total_modules"
    echo "  Three.js modules:           $three_modules"
    echo "  Shared utility modules:     $shared_count"
    echo "  Sharing overhead:           ${impact}%"
    
    # Memory usage estimation
    if [ $total_modules -gt 0 ]; then
        checks_per_module=$(( three_modules + shared_count ))
        total_checks=$(( total_modules * checks_per_module ))
        echo ""
        echo "  Estimated descendant checks: ~$total_checks"
        echo "  (Each module checks against $checks_per_module shared modules)"
    fi
fi

# Clean up
rm -f rspack.config.multi-app-*.cjs multi-app-stats.json
rm -rf app1-viz app2-editor app3-game shared-three multi-app-entry.js

cd ../..

echo ""
echo "✅ Multi-app Three.js benchmark complete!"
echo ""
echo "💡 This benchmark simulates a realistic scenario where multiple"
echo "   applications share Three.js and custom utility modules."
echo ""
echo "🎯 Key findings:"
echo "   - Multiple apps increase the complexity of sharing analysis"
echo "   - Each app's modules must check against all shared modules"
echo "   - The is_consume_shared_descendant function is called O(n*m) times"
echo "   - Performance degradation is more visible with complex sharing"