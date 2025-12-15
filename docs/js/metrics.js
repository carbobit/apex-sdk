/**
 * Real-time Blockchain Metrics
 * Displays live network statistics with smooth animations
 */

class BlockchainMetrics {
    constructor() {
        this.metrics = {
            tps: { element: null, value: 0, target: 0 },
            blocks: { element: null, value: 0, target: 0 },
            chains: { element: null, value: 10 },
            uptime: { element: null, value: 99.9 }
        };
        
        this.init();
    }
    
    init() {
        // Find metric elements
        this.metrics.tps.element = document.getElementById('metric-tps');
        this.metrics.blocks.element = document.getElementById('metric-blocks');
        this.metrics.chains.element = document.getElementById('metric-chains');
        this.metrics.uptime.element = document.getElementById('metric-uptime');
        
        // Start animations
        this.animate();
        this.updateMetrics();
    }
    
    animate() {
        // Smooth number transitions
        Object.keys(this.metrics).forEach(key => {
            const metric = this.metrics[key];
            if (!metric.element) return;
            
            // Interpolate towards target
            const diff = metric.target - metric.value;
            metric.value += diff * 0.1;
            
            // Update display
            if (key === 'uptime') {
                metric.element.textContent = metric.value.toFixed(1) + '%';
            } else if (key === 'chains') {
                metric.element.textContent = metric.value + '+';
            } else {
                metric.element.textContent = Math.floor(metric.value).toLocaleString();
            }
        });
        
        requestAnimationFrame(() => this.animate());
    }
    
    updateMetrics() {
        // Simulate real-time updates
        setInterval(() => {
            // TPS: 500-1500 range
            this.metrics.tps.target = 500 + Math.random() * 1000;
            
            // Blocks: increment steadily
            this.metrics.blocks.target += Math.floor(Math.random() * 5 + 1);
        }, 2000);
        
        // Simulate network events
        setInterval(() => {
            // Occasional spikes in TPS
            if (Math.random() > 0.7) {
                this.metrics.tps.target = 1500 + Math.random() * 500;
            }
        }, 5000);
    }
    
    // Public method to update metrics from real API
    updateFromAPI(data) {
        if (data.tps !== undefined) this.metrics.tps.target = data.tps;
        if (data.blocks !== undefined) this.metrics.blocks.target = data.blocks;
        if (data.chains !== undefined) this.metrics.chains.value = data.chains;
        if (data.uptime !== undefined) this.metrics.uptime.value = data.uptime;
    }
}

// Initialize metrics
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.blockchainMetrics = new BlockchainMetrics();
    });
} else {
    window.blockchainMetrics = new BlockchainMetrics();
}
