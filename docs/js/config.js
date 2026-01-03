/**
 * Configuration Module
 * Centralized configuration for the documentation site
 */

const CONFIG = {
    // SEO Configuration
    seo: {
        siteName: 'Apex SDK Protocol',
        siteDescription: 'Unified Rust SDK for Substrate and EVM blockchain development. Build secure cross-chain applications with compile-time safety and native performance.',
        keywords: [
            'Apex SDK Protocol',
            'Rust blockchain SDK',
            'Substrate SDK',
            'EVM SDK',
            'cross-chain development',
            'Polkadot SDK',
            'Ethereum SDK',
            'blockchain development',
            'Web3 Rust',
            'Kusama SDK',
            'smart contract SDK',
            'DeFi development',
            'blockchain integration',
            'compile-time safety',
            'type-safe blockchain',
            'multi-chain SDK',
            'Rust Web3',
            'Substrate framework',
            'EVM compatibility',
            'cross-chain protocol'
        ],
        author: 'Apex SDK Team',
        canonicalUrl: 'https://apexsdk.dev',
        ogImage: 'https://apexsdk.dev/assets/og-image.png'
    },
    
    // API Endpoints
    api: {
        metrics: null, // To be configured for real API
        sandbox: null  // To be configured for code execution
    },

    // Web3Forms Configuration
    web3forms: {
        accessKey: (() => {
            // Build-time placeholder (replaced by Cloudflare Pages build)
            const buildTimeKey = '__WEB3FORMS_ACCESS_KEY__';

            // Try multiple sources for the access key
            const key =
                // 1. From build-time injection (Cloudflare Pages)
                (buildTimeKey && !buildTimeKey.startsWith('__WEB3FORMS')) ? buildTimeKey :
                // 2. From window global (set by build process)
                window.WEB3FORMS_ACCESS_KEY ||
                // 3. From meta tag
                document.querySelector('meta[name="web3forms-key"]')?.content ||
                // 4. From environment (if using bundler)
                (typeof process !== 'undefined' && process.env?.WEB3FORMS_ACCESS_KEY) ||
                // 5. From local storage (for development)
                localStorage.getItem('web3forms-key') ||
                null;

            if (!key) {
                console.warn('Web3Forms access key not found. Contact form will be disabled.');
            }

            return key;
        })(),
        endpoint: 'https://api.web3forms.com/submit',
        enabled: function() {
            return !!this.accessKey;
        }
    },
    
    // Animation Settings
    animations: {
        enabled: true,
        reducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
        duration: {
            fast: 0.3,
            base: 0.6,
            slow: 1.0
        }
    },
    
    // Personalization
    personalization: {
        enabled: true,
        storageKey: 'apex-sdk-profile',
        recommendationDelay: 5000
    },
    
    // Feature Flags
    features: {
        codeEditor: true,
        blockchainViz: true,
        metrics: true,
        workflowSimulator: true,
        personalization: true
    }
};

// Expose globally for all scripts
if (typeof window !== 'undefined') {
    window.CONFIG = CONFIG;
}
