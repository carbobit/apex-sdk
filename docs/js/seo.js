/**
 * SEO Module
 * Handles all SEO-related functionality including meta tags and structured data
 */

// Get CONFIG from window (set by config.js)
const CONFIG = (typeof window !== 'undefined' && window.CONFIG) || {};

export class SEOManager {
    constructor() {
        this.init();
    }
    
    init() {
        this.updateMetaTags();
        this.addStructuredData();
        this.optimizeForApex();
    }
    
    updateMetaTags() {
        const seo = CONFIG.seo;
        
        // Update or create meta tags
        this.setMetaTag('name', 'description', seo.siteDescription);
        this.setMetaTag('name', 'keywords', seo.keywords.join(', '));
        this.setMetaTag('name', 'author', seo.author);
        this.setMetaTag('property', 'og:title', seo.siteName);
        this.setMetaTag('property', 'og:description', seo.siteDescription);
        this.setMetaTag('property', 'og:image', seo.ogImage);
        this.setMetaTag('property', 'og:url', seo.canonicalUrl);
        this.setMetaTag('property', 'og:type', 'website');
        this.setMetaTag('name', 'twitter:card', 'summary_large_image');
        this.setMetaTag('name', 'twitter:title', seo.siteName);
        this.setMetaTag('name', 'twitter:description', seo.siteDescription);
        this.setMetaTag('name', 'twitter:image', seo.ogImage);
    }
    
    setMetaTag(attribute, name, content) {
        let meta = document.querySelector(`meta[${attribute}="${name}"]`);
        if (!meta) {
            meta = document.createElement('meta');
            meta.setAttribute(attribute, name);
            document.head.appendChild(meta);
        }
        meta.setAttribute('content', content);
    }
    
    optimizeForApex() {
        // Add specific terms to differentiate from other "Apex" products
        const specificTerms = [
            'Apex SDK Protocol',
            'Apex SDK blockchain',
            'Apex SDK Rust',
            'Apex SDK Substrate',
            'Apex SDK EVM',
            'Apex SDK cross-chain'
        ];
        
        // Update title to be more specific
        const currentTitle = document.title;
        if (!currentTitle.includes('Protocol') && !currentTitle.includes('blockchain')) {
            document.title = `Apex SDK Protocol - ${currentTitle}`;
        }
        
        // Add specific keywords to meta
        const keywordsMeta = document.querySelector('meta[name="keywords"]');
        if (keywordsMeta) {
            const existingKeywords = keywordsMeta.getAttribute('content') || '';
            const newKeywords = [...specificTerms, ...existingKeywords.split(', ')].join(', ');
            keywordsMeta.setAttribute('content', newKeywords);
        }
    }
    
    addStructuredData() {
        const seo = CONFIG.seo;
        
        // SoftwareApplication schema
        const softwareAppSchema = {
            "@context": "https://schema.org",
            "@type": "SoftwareApplication",
            "name": seo.siteName,
            "applicationCategory": "DeveloperApplication",
            "operatingSystem": "Cross-platform",
            "description": seo.siteDescription,
            "url": seo.canonicalUrl,
            "offers": {
                "@type": "Offer",
                "price": "0",
                "priceCurrency": "USD"
            },
            "author": {
                "@type": "Organization",
                "name": "Apex SDK Team",
                "url": "https://github.com/eurybits/apex-sdk"
            },
            "programmingLanguage": "Rust",
            "keywords": seo.keywords.join(', '),
            "softwareVersion": "0.1.4",
            "license": "https://www.apache.org/licenses/LICENSE-2.0",
            "codeRepository": "https://github.com/eurybits/apex-sdk",
            "alternateName": "Apex SDK Protocol",
            "brand": {
                "@type": "Brand",
                "name": "Apex SDK Protocol"
            }
        };
        
        // WebSite schema with search
        const websiteSchema = {
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": seo.siteName,
            "url": seo.canonicalUrl,
            "description": seo.siteDescription,
            "publisher": {
                "@type": "Organization",
                "name": "Apex SDK Team",
                "logo": {
                    "@type": "ImageObject",
                    "url": seo.ogImage
                }
            },
            "potentialAction": {
                "@type": "SearchAction",
                "target": {
                    "@type": "EntryPoint",
                    "urlTemplate": `${seo.canonicalUrl}/viewer.html?q={search_term_string}`
                },
                "query-input": "required name=search_term_string"
            },
            "alternateName": [
                "Apex SDK Protocol",
                "Apex SDK blockchain",
                "Apex SDK Rust"
            ]
        };
        
        // TechArticle schema
        const techArticleSchema = {
            "@context": "https://schema.org",
            "@type": "TechArticle",
            "headline": `${seo.siteName} - Cross-Chain Blockchain Development`,
            "description": seo.siteDescription,
            "author": {
                "@type": "Organization",
                "name": "Apex SDK Team"
            },
            "datePublished": "2025-01-01",
            "dateModified": new Date().toISOString().split('T')[0],
            "image": seo.ogImage,
            "articleSection": "Blockchain Development",
            "keywords": seo.keywords,
            "dependencies": "Rust 1.85+",
            "proficiencyLevel": "Beginner to Advanced",
            "about": {
                "@type": "Thing",
                "name": "Blockchain Development",
                "description": "Cross-chain blockchain application development using Rust"
            }
        };
        
        // Remove existing schemas
        document.querySelectorAll('script[type="application/ld+json"]').forEach(el => el.remove());
        
        // Add new schemas
        [softwareAppSchema, websiteSchema, techArticleSchema].forEach(schema => {
            const script = document.createElement('script');
            script.type = 'application/ld+json';
            script.textContent = JSON.stringify(schema);
            document.head.appendChild(script);
        });
    }
    
    updatePageSEO(title, description, keywords = []) {
        document.title = `${title} | ${CONFIG.seo.siteName}`;
        this.setMetaTag('name', 'description', description);
        this.setMetaTag('property', 'og:title', title);
        this.setMetaTag('property', 'og:description', description);
        this.setMetaTag('name', 'twitter:title', title);
        this.setMetaTag('name', 'twitter:description', description);
        
        if (keywords.length > 0) {
            const allKeywords = [...CONFIG.seo.keywords, ...keywords].join(', ');
            this.setMetaTag('name', 'keywords', allKeywords);
        }
    }
}

// Initialize SEO on load
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.seoManager = new SEOManager();
    });
} else {
    window.seoManager = new SEOManager();
}
