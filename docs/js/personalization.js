/**
 * Adaptive Personalization System
 * Dynamically tailors content based on user interaction patterns
 * Now renders actual markdown content in recommendations
 */

// Get CONFIG from window (set by config.js)
const CONFIG = (typeof window !== 'undefined' && window.CONFIG) || {
    personalization: {
        enabled: true,
        storageKey: 'apex-sdk-profile',
        recommendationDelay: 5000
    }
};

class PersonalizationEngine {
    constructor() {
        this.userProfile = {
            expertise: 'beginner',
            interests: [],
            visitedSections: [],
            timeOnPage: 0,
            interactions: 0
        };
        
        this.contentPaths = {
            beginner: [
                { file: 'QUICK_START.md', title: 'Quick Start Guide', description: 'Get started with Apex SDK in minutes' },
                { file: 'CLI_GUIDE.md', title: 'CLI Guide', description: 'Master the command-line interface' },
                { file: 'TESTNETS.md', title: 'Testnets', description: 'Test your applications on test networks' }
            ],
            intermediate: [
                { file: 'API.md', title: 'API Reference', description: 'Complete API documentation' },
                { file: 'SYSTEM_ARCHITECTURE.md', title: 'System Architecture', description: 'Understand the SDK architecture' },
                { file: 'Substrate_Features.md', title: 'Substrate Features', description: 'Substrate-specific capabilities' }
            ],
            advanced: [
                { file: 'TYPED_METADATA.md', title: 'Typed Metadata', description: 'Type-safe runtime interaction' },
                { file: 'TESTING_FRAMEWORK.md', title: 'Testing Framework', description: 'Write comprehensive tests' },
                { file: 'SECURITY_AUDIT.md', title: 'Security Audit', description: 'Security audit results' }
            ]
        };
        
        this.marked = null; // Will be loaded dynamically
        this.init();
    }
    
    async init() {
        this.loadProfile();
        this.trackInteractions();
        await this.loadMarked();
        this.updateContentRecommendations();
    }
    
    async loadMarked() {
        if (typeof marked !== 'undefined') {
            this.marked = marked;
            return;
        }
        
        // Load marked.js if not already loaded
        return new Promise((resolve, reject) => {
            const script = document.createElement('script');
            script.src = 'https://cdnjs.cloudflare.com/ajax/libs/marked/11.1.1/marked.min.js';
            script.onload = () => {
                this.marked = marked;
                resolve();
            };
            script.onerror = reject;
            document.head.appendChild(script);
        });
    }
    
    loadProfile() {
        const saved = localStorage.getItem(CONFIG.personalization.storageKey);
        if (saved) {
            this.userProfile = { ...this.userProfile, ...JSON.parse(saved) };
        }
    }
    
    saveProfile() {
        localStorage.setItem(CONFIG.personalization.storageKey, JSON.stringify(this.userProfile));
    }
    
    trackInteractions() {
        const sections = document.querySelectorAll('section[id]');
        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    const sectionId = entry.target.id;
                    if (!this.userProfile.visitedSections.includes(sectionId)) {
                        this.userProfile.visitedSections.push(sectionId);
                        this.userProfile.interactions++;
                        this.updateExpertiseLevel();
                        this.saveProfile();
                    }
                }
            });
        }, { threshold: 0.5 });
        
        sections.forEach(section => observer.observe(section));
        
        setInterval(() => {
            this.userProfile.timeOnPage += 1;
            if (this.userProfile.timeOnPage % 30 === 0) {
                this.saveProfile();
            }
        }, 1000);
    }
    
    updateExpertiseLevel() {
        if (this.userProfile.interactions < 3) {
            this.userProfile.expertise = 'beginner';
        } else if (this.userProfile.interactions < 10) {
            this.userProfile.expertise = 'intermediate';
        } else {
            this.userProfile.expertise = 'advanced';
        }
    }
    
    async updateContentRecommendations() {
        const recommendations = this.contentPaths[this.userProfile.expertise];
        await this.showRecommendations(recommendations);
    }
    
    async loadMarkdownPreview(file) {
        try {
            const response = await fetch(file);
            if (!response.ok) throw new Error('Failed to load');
            const markdown = await response.text();
            
            // Extract first paragraph or first few lines
            const lines = markdown.split('\n').filter(line => line.trim());
            let preview = '';
            
            // Get first paragraph (non-header, non-code)
            for (const line of lines) {
                if (line.trim() && !line.startsWith('#') && !line.startsWith('```') && !line.startsWith('|')) {
                    preview = line.substring(0, 150);
                    if (preview.length < line.length) preview += '...';
                    break;
                }
            }
            
            return preview || 'Click to read more';
        } catch (error) {
            console.error('Error loading markdown preview:', error);
            return 'Click to read more';
        }
    }
    
    async showRecommendations(recommendations) {
        let panel = document.getElementById('personalized-recommendations');
        if (!panel) {
            panel = document.createElement('div');
            panel.id = 'personalized-recommendations';
            panel.className = 'recommendations-panel';
            panel.style.cssText = `
                position: fixed;
                bottom: 2rem;
                right: 2rem;
                background: var(--bg-glass);
                backdrop-filter: blur(20px);
                border: 1px solid var(--border-color);
                border-radius: var(--radius-lg);
                padding: 1.5rem;
                max-width: 400px;
                max-height: 600px;
                overflow-y: auto;
                box-shadow: var(--shadow-glass);
                z-index: 1000;
                display: none;
            `;
            document.body.appendChild(panel);
        }
        
        const title = document.createElement('h3');
        title.textContent = 'Recommended for You';
        title.style.cssText = 'font-size: 1.25rem; margin-bottom: 1rem; color: var(--text-primary); font-weight: 700;';
        
        const list = document.createElement('div');
        list.style.cssText = 'display: flex; flex-direction: column; gap: 1rem;';
        
        // Load and render each recommendation
        for (const rec of recommendations.slice(0, 3)) {
            const card = document.createElement('div');
            card.className = 'recommendation-card';
            card.style.cssText = `
                background: var(--bg-tertiary);
                border: 1px solid var(--border-color);
                border-radius: var(--radius-md);
                padding: 1rem;
                cursor: pointer;
                transition: all 0.3s ease;
            `;
            
            card.addEventListener('mouseenter', () => {
                card.style.transform = 'translateY(-2px)';
                card.style.borderColor = 'var(--primary)';
                card.style.boxShadow = 'var(--shadow-glow)';
            });
            
            card.addEventListener('mouseleave', () => {
                card.style.transform = 'translateY(0)';
                card.style.borderColor = 'var(--border-color)';
                card.style.boxShadow = 'none';
            });
            
            card.addEventListener('click', () => {
                window.location.href = `viewer.html?doc=${rec.file}`;
            });
            
            const titleEl = document.createElement('h4');
            titleEl.textContent = rec.title;
            titleEl.style.cssText = 'font-size: 1rem; margin-bottom: 0.5rem; color: var(--text-primary); font-weight: 600;';
            
            const descEl = document.createElement('p');
            descEl.textContent = rec.description;
            descEl.style.cssText = 'font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 0.5rem; line-height: 1.5;';
            
            const previewEl = document.createElement('p');
            previewEl.className = 'recommendation-preview';
            previewEl.style.cssText = 'font-size: 0.8125rem; color: var(--text-tertiary); font-style: italic; margin-top: 0.5rem;';
            previewEl.textContent = 'Loading preview...';
            
            card.appendChild(titleEl);
            card.appendChild(descEl);
            card.appendChild(previewEl);
            list.appendChild(card);
            
            // Load preview asynchronously
            this.loadMarkdownPreview(rec.file).then(preview => {
                previewEl.textContent = preview;
            });
        }
        
        const closeBtn = document.createElement('button');
        closeBtn.textContent = '×';
        closeBtn.style.cssText = `
            position: absolute;
            top: 0.5rem;
            right: 0.5rem;
            background: none;
            border: none;
            color: var(--text-secondary);
            font-size: 1.5rem;
            cursor: pointer;
            width: 2rem;
            height: 2rem;
            display: flex;
            align-items: center;
            justify-content: center;
            border-radius: var(--radius-sm);
            transition: all 0.2s ease;
        `;
        closeBtn.addEventListener('mouseenter', () => {
            closeBtn.style.background = 'var(--bg-tertiary)';
            closeBtn.style.color = 'var(--text-primary)';
        });
        closeBtn.addEventListener('click', () => {
            panel.style.display = 'none';
        });
        
        panel.innerHTML = '';
        panel.appendChild(closeBtn);
        panel.appendChild(title);
        panel.appendChild(list);
        
        // Show panel after delay
        setTimeout(() => {
            if (CONFIG.personalization.enabled) {
                panel.style.display = 'block';
                if (typeof gsap !== 'undefined') {
                    gsap.from(panel, { opacity: 0, y: 20, duration: 0.5 });
                }
            }
        }, CONFIG.personalization.recommendationDelay);
    }
    
    setExpertiseLevel(level) {
        this.userProfile.expertise = level;
        this.saveProfile();
        this.updateContentRecommendations();
    }
}

// Initialize personalization
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.personalizationEngine = new PersonalizationEngine();
    });
} else {
    window.personalizationEngine = new PersonalizationEngine();
}

// Export for potential module usage
if (typeof module !== 'undefined' && module.exports) {
    module.exports = PersonalizationEngine;
}