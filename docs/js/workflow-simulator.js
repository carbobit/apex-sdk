/**
 * Animated Workflow Simulator
 * Illustrates protocol interactions and transaction flows
 */

class WorkflowSimulator {
    constructor(containerId) {
        this.container = document.getElementById(containerId);
        if (!this.container) return;
        
        this.canvas = null;
        this.ctx = null;
        this.nodes = [];
        this.connections = [];
        this.animationId = null;
        this.currentStep = 0;
        this.steps = [
            { name: 'Initialize SDK', color: '#3b82f6', x: 100, y: 100 },
            { name: 'Build Transaction', color: '#8b5cf6', x: 300, y: 100 },
            { name: 'Sign Transaction', color: '#06b6d4', x: 500, y: 100 },
            { name: 'Submit to Network', color: '#10b981', x: 700, y: 100 },
            { name: 'Confirm Block', color: '#f59e0b', x: 900, y: 100 }
        ];
        
        this.init();
    }
    
    init() {
        this.canvas = document.createElement('canvas');
        this.canvas.width = this.container.clientWidth || 1000;
        this.canvas.height = 300;
        this.canvas.style.width = '100%';
        this.canvas.style.height = '100%';
        this.ctx = this.canvas.getContext('2d');
        this.container.appendChild(this.canvas);
        
        this.resize();
        window.addEventListener('resize', () => this.resize());
        
        this.createNodes();
        this.animate();
        this.startSimulation();
    }
    
    createNodes() {
        const width = this.canvas.width;
        const spacing = width / (this.steps.length + 1);
        
        this.steps.forEach((step, i) => {
            step.x = spacing * (i + 1);
            step.y = this.canvas.height / 2;
        });
    }
    
    resize() {
        const rect = this.container.getBoundingClientRect();
        this.canvas.width = rect.width;
        this.canvas.height = 300;
        this.createNodes();
    }
    
    draw() {
        // Clear canvas
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        
        // Draw connections
        for (let i = 0; i < this.steps.length - 1; i++) {
            const start = this.steps[i];
            const end = this.steps[i + 1];
            const progress = Math.max(0, Math.min(1, (this.currentStep - i) / 1));
            
            // Draw line
            this.ctx.strokeStyle = i < this.currentStep ? start.color : 'rgba(255, 255, 255, 0.1)';
            this.ctx.lineWidth = 3;
            this.ctx.beginPath();
            this.ctx.moveTo(start.x, start.y);
            this.ctx.lineTo(end.x, end.y);
            this.ctx.stroke();
            
            // Draw animated pulse
            if (i === Math.floor(this.currentStep) && progress > 0 && progress < 1) {
                const pulseX = start.x + (end.x - start.x) * progress;
                const pulseY = start.y + (end.y - start.y) * progress;
                
                const gradient = this.ctx.createRadialGradient(pulseX, pulseY, 0, pulseX, pulseY, 20);
                gradient.addColorStop(0, start.color + '80');
                gradient.addColorStop(1, start.color + '00');
                
                this.ctx.fillStyle = gradient;
                this.ctx.beginPath();
                this.ctx.arc(pulseX, pulseY, 20, 0, Math.PI * 2);
                this.ctx.fill();
            }
        }
        
        // Draw nodes
        this.steps.forEach((step, i) => {
            const isActive = i <= this.currentStep;
            const isCurrent = Math.floor(this.currentStep) === i;
            
            // Outer glow for active nodes
            if (isActive) {
                const glowGradient = this.ctx.createRadialGradient(step.x, step.y, 0, step.x, step.y, 40);
                glowGradient.addColorStop(0, step.color + '40');
                glowGradient.addColorStop(1, step.color + '00');
                this.ctx.fillStyle = glowGradient;
                this.ctx.beginPath();
                this.ctx.arc(step.x, step.y, 40, 0, Math.PI * 2);
                this.ctx.fill();
            }
            
            // Node circle
            this.ctx.fillStyle = isActive ? step.color : 'rgba(255, 255, 255, 0.1)';
            this.ctx.beginPath();
            this.ctx.arc(step.x, step.y, 25, 0, Math.PI * 2);
            this.ctx.fill();
            
            // Inner circle for current step
            if (isCurrent) {
                const pulseSize = 15 + Math.sin(Date.now() * 0.005) * 5;
                this.ctx.fillStyle = '#ffffff';
                this.ctx.beginPath();
                this.ctx.arc(step.x, step.y, pulseSize, 0, Math.PI * 2);
                this.ctx.fill();
            }
            
            // Label
            this.ctx.fillStyle = isActive ? '#ffffff' : 'rgba(255, 255, 255, 0.5)';
            this.ctx.font = '14px Inter, sans-serif';
            this.ctx.textAlign = 'center';
            this.ctx.textBaseline = 'top';
            this.ctx.fillText(step.name, step.x, step.y + 35);
        });
    }
    
    animate() {
        this.draw();
        this.animationId = requestAnimationFrame(() => this.animate());
    }
    
    startSimulation() {
        setInterval(() => {
            this.currentStep += 0.02;
            if (this.currentStep >= this.steps.length) {
                this.currentStep = 0;
            }
        }, 50);
    }
    
    reset() {
        this.currentStep = 0;
    }
    
    goToStep(stepIndex) {
        this.currentStep = stepIndex;
    }
    
    destroy() {
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
        }
    }
}

// Initialize workflow simulator
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        const container = document.getElementById('workflow-simulator');
        if (container) {
            window.workflowSimulator = new WorkflowSimulator('workflow-simulator');
        }
    });
} else {
    const container = document.getElementById('workflow-simulator');
    if (container) {
        window.workflowSimulator = new WorkflowSimulator('workflow-simulator');
    }
}
