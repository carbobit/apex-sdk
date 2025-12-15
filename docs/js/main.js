/**
 * Main Application Module
 * Orchestrates all modules and initializes the application
 */

// Get CONFIG from window (set by config.js)
const CONFIG = (typeof window !== 'undefined' && window.CONFIG) || {
    features: {
        codeEditor: true,
        blockchainViz: true,
        metrics: true,
        workflowSimulator: true,
        personalization: true
    },
    animations: {
        enabled: true,
        reducedMotion: false
    }
};

class App {
    constructor() {
        this.modules = {};
        this.init();
    }
    
    async init() {
        // Initialize SEO first
        this.seoManager = new SEOManager();
        
        // Initialize modules based on feature flags
        if (CONFIG.features.codeEditor) {
            await this.initCodeEditor();
        }
        
        // Initialize animations
        this.initAnimations();
        
        // Initialize navigation
        this.initNavigation();
    }
    
    async initCodeEditor() {
        // Code editor initialization will be handled by inline script
        // This is a placeholder for future modularization
        return Promise.resolve();
    }
    
    initAnimations() {
        if (!CONFIG.animations.enabled || CONFIG.animations.reducedMotion) {
            return;
        }
        
        if (typeof gsap !== 'undefined' && typeof ScrollTrigger !== 'undefined') {
            gsap.registerPlugin(ScrollTrigger);
            this.setupScrollAnimations();
        }
    }
    
    setupScrollAnimations() {
        gsap.utils.toArray('.feature-card, .arch-layer').forEach((el, i) => {
            gsap.from(el, {
                scrollTrigger: {
                    trigger: el,
                    start: 'top 80%',
                    toggleActions: 'play none none reverse'
                },
                opacity: 0,
                y: 50,
                duration: 0.8,
                delay: i * 0.1
            });
        });
        
        // Parallax effect for hero
        const heroContent = document.querySelector('.hero-content');
        if (heroContent) {
            gsap.to(heroContent, {
                scrollTrigger: {
                    trigger: '.hero',
                    start: 'top top',
                    end: 'bottom top',
                    scrub: true
                },
                y: 100,
                opacity: 0
            });
        }
    }
    
    initNavigation() {
        // Navbar scroll effect
        const nav = document.getElementById('navbar');
        if (nav) {
            window.addEventListener('scroll', () => {
                if (window.scrollY > 50) {
                    nav.classList.add('scrolled');
                } else {
                    nav.classList.remove('scrolled');
                }
            });
        }
        
        // Smooth scroll for anchor links
        document.querySelectorAll('a[href^="#"]').forEach(anchor => {
            anchor.addEventListener('click', function (e) {
                e.preventDefault();
                const target = document.querySelector(this.getAttribute('href'));
                if (target) {
                    target.scrollIntoView({
                        behavior: 'smooth',
                        block: 'start'
                    });
                }
            });
        });
    }
}

// Initialize app
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.app = new App();
    });
} else {
    window.app = new App();
}

export default App;
