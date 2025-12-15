/**
 * Advanced 3D Blockchain Visualization
 * Creates an immersive 3D representation of blockchain networks
 */

class BlockchainVisualization {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        if (!this.container) return;
        
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.nodes = [];
        this.connections = [];
        this.animationId = null;
        
        this.init();
    }
    
    init() {
        // Scene setup
        this.scene = new THREE.Scene();
        this.scene.background = null;
        
        // Camera setup
        const aspect = this.container.clientWidth / this.container.clientHeight;
        this.camera = new THREE.PerspectiveCamera(75, aspect, 0.1, 1000);
        this.camera.position.set(0, 8, 12);
        this.camera.lookAt(0, 0, 0);
        
        // Renderer setup
        this.renderer = new THREE.WebGLRenderer({ 
            alpha: true, 
            antialias: true,
            powerPreference: "high-performance"
        });
        this.renderer.setSize(this.container.clientWidth, this.container.clientHeight);
        this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        this.container.appendChild(this.renderer.domElement);
        
        // Lighting
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
        this.scene.add(ambientLight);
        
        const directionalLight = new THREE.DirectionalLight(0x3b82f6, 1);
        directionalLight.position.set(5, 10, 5);
        this.scene.add(directionalLight);
        
        // Create blockchain network
        this.createNetwork();
        
        // Handle resize
        window.addEventListener('resize', () => this.handleResize());
        
        // Start animation
        this.animate();
    }
    
    createNetwork() {
        const nodeCount = 12;
        const radius = 4;
        const colors = [0x3b82f6, 0x8b5cf6, 0x06b6d4];
        
        // Create nodes (blockchain nodes)
        for (let i = 0; i < nodeCount; i++) {
            const geometry = new THREE.IcosahedronGeometry(0.2, 0);
            const material = new THREE.MeshPhongMaterial({
                color: colors[i % colors.length],
                emissive: colors[i % colors.length],
                emissiveIntensity: 0.3,
                transparent: true,
                opacity: 0.9
            });
            
            const node = new THREE.Mesh(geometry, material);
            
            const angle = (i / nodeCount) * Math.PI * 2;
            node.position.x = Math.cos(angle) * radius;
            node.position.z = Math.sin(angle) * radius;
            node.position.y = (Math.sin(i * 0.5) - 0.5) * 2;
            
            // Store original position for animation
            node.userData = {
                baseY: node.position.y,
                angle: angle,
                speed: 0.5 + Math.random() * 0.5
            };
            
            this.scene.add(node);
            this.nodes.push(node);
        }
        
        // Create connections (blockchain links)
        const lineMaterial = new THREE.LineBasicMaterial({
            color: 0x3b82f6,
            transparent: true,
            opacity: 0.2
        });
        
        // Connect each node to its neighbors
        this.nodes.forEach((node, i) => {
            const nextIndex = (i + 1) % this.nodes.length;
            const nextNode = this.nodes[nextIndex];
            
            const geometry = new THREE.BufferGeometry().setFromPoints([
                new THREE.Vector3(node.position.x, node.position.y, node.position.z),
                new THREE.Vector3(nextNode.position.x, nextNode.position.y, nextNode.position.z)
            ]);
            
            const line = new THREE.Line(geometry, lineMaterial);
            this.scene.add(line);
            this.connections.push(line);
        });
        
        // Add cross-connections for network effect
        for (let i = 0; i < nodeCount; i += 3) {
            const targetIndex = (i + 4) % nodeCount;
            const node1 = this.nodes[i];
            const node2 = this.nodes[targetIndex];
            
            const geometry = new THREE.BufferGeometry().setFromPoints([
                new THREE.Vector3(node1.position.x, node1.position.y, node1.position.z),
                new THREE.Vector3(node2.position.x, node2.position.y, node2.position.z)
            ]);
            
            const line = new THREE.Line(geometry, lineMaterial);
            this.scene.add(line);
            this.connections.push(line);
        }
    }
    
    animate() {
        this.animationId = requestAnimationFrame(() => this.animate());
        
        const time = Date.now() * 0.001;
        
        // Animate nodes
        this.nodes.forEach((node, i) => {
            // Rotate nodes
            node.rotation.x += 0.01;
            node.rotation.y += 0.01;
            
            // Float animation
            node.position.y = node.userData.baseY + Math.sin(time * node.userData.speed + i) * 0.5;
            
            // Pulse effect
            const scale = 1 + Math.sin(time * 2 + i) * 0.1;
            node.scale.set(scale, scale, scale);
        });
        
        // Update connections
        this.connections.forEach((line, i) => {
            const positions = line.geometry.attributes.position;
            if (positions && positions.count >= 2) {
                const start = new THREE.Vector3().fromBufferAttribute(positions, 0);
                const end = new THREE.Vector3().fromBufferAttribute(positions, 1);
                
                // Find corresponding nodes
                const startNode = this.nodes.find(n => 
                    Math.abs(n.position.x - start.x) < 0.1 &&
                    Math.abs(n.position.z - start.z) < 0.1
                );
                const endNode = this.nodes.find(n => 
                    Math.abs(n.position.x - end.x) < 0.1 &&
                    Math.abs(n.position.z - end.z) < 0.1
                );
                
                if (startNode && endNode) {
                    positions.setXYZ(0, startNode.position.x, startNode.position.y, startNode.position.z);
                    positions.setXYZ(1, endNode.position.x, endNode.position.y, endNode.position.z);
                    positions.needsUpdate = true;
                }
            }
        });
        
        // Rotate camera slightly for dynamic view
        const cameraRadius = 12;
        this.camera.position.x = Math.cos(time * 0.1) * cameraRadius;
        this.camera.position.z = Math.sin(time * 0.1) * cameraRadius;
        this.camera.lookAt(0, 0, 0);
        
        this.renderer.render(this.scene, this.camera);
    }
    
    handleResize() {
        if (!this.container) return;
        
        const width = this.container.clientWidth;
        const height = this.container.clientHeight;
        
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(width, height);
    }
    
    destroy() {
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
        }
        
        // Clean up Three.js resources
        this.nodes.forEach(node => {
            node.geometry.dispose();
            node.material.dispose();
        });
        
        this.connections.forEach(line => {
            line.geometry.dispose();
            line.material.dispose();
        });
        
        if (this.renderer) {
            this.renderer.dispose();
        }
    }
}

// Initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.blockchainViz = new BlockchainVisualization('blockchain-viz');
    });
} else {
    window.blockchainViz = new BlockchainVisualization('blockchain-viz');
}
