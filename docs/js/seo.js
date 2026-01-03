// SEO and Structured Data Management
document.addEventListener('DOMContentLoaded', () => {
    updateStructuredData();
});

function updateStructuredData() {
    const structuredData = {
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "Apex SDK Protocol",
        "softwareVersion": "0.1.5",
        "applicationCategory": "DeveloperApplication",
        "operatingSystem": "Cross-platform",
        "description": "Unified Rust SDK for Substrate & EVM blockchain development.",
        "offers": {
            "@type": "Offer",
            "price": "0",
            "priceCurrency": "USD"
        }
    };

    const script = document.createElement('script');
    script.type = 'application/ld+json';
    script.text = JSON.stringify(structuredData);
    document.head.appendChild(script);
}
