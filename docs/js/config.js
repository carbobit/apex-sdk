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
