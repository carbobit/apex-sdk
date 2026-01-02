document.addEventListener('DOMContentLoaded', () => {
    initThemeToggle();
    initMobileMenu();
    initScrollEffects();
    renderChains();
    initChainsFilter();
    setCurrentYear();
});

function setCurrentYear() {
    const yearElement = document.getElementById('current-year');
    if (yearElement) {
        yearElement.textContent = new Date().getFullYear();
    }
}

// Theme Toggle
function initThemeToggle() {
    const toggle = document.getElementById('themeToggle');
    const html = document.documentElement;
    const iconMoon = document.getElementById('theme-icon-moon');
    const iconSun = document.getElementById('theme-icon-sun');

    if (!toggle) return;

    const savedTheme = localStorage.getItem('theme') || 'dark';
    setTheme(savedTheme);

    toggle.addEventListener('click', () => {
        const currentTheme = html.getAttribute('data-theme');
        const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
        setTheme(newTheme);
    });

    function setTheme(theme) {
        html.setAttribute('data-theme', theme);
        localStorage.setItem('theme', theme);

        if (theme === 'dark') {
            iconMoon.style.display = 'none';
            iconSun.style.display = 'block';
        } else {
            iconMoon.style.display = 'block';
            iconSun.style.display = 'none';
        }
    }
}

// Mobile Menu
function initMobileMenu() {
    const toggle = document.getElementById('mobileMenuToggle');
    const menu = document.getElementById('navMenu');

    if (!toggle || !menu) return;

    toggle.addEventListener('click', () => {
        const isExpanded = toggle.getAttribute('aria-expanded') === 'true';
        toggle.setAttribute('aria-expanded', !isExpanded);
        menu.classList.toggle('active');

        const lines = toggle.querySelectorAll('.hamburger-line');
        if (!isExpanded) {
            lines[0].style.transform = 'rotate(45deg) translate(5px, 5px)';
            lines[1].style.opacity = '0';
            lines[2].style.transform = 'rotate(-45deg) translate(5px, -5px)';
        } else {
            lines[0].style.transform = 'none';
            lines[1].style.opacity = '1';
            lines[2].style.transform = 'none';
        }
    });

    menu.querySelectorAll('a').forEach(link => {
        link.addEventListener('click', () => {
            if (menu.classList.contains('active')) {
                toggle.click();
            }
        });
    });
}

// Scroll Effects
function initScrollEffects() {
    const navbar = document.getElementById('navbar');

    window.addEventListener('scroll', () => {
        if (window.scrollY > 20) {
            navbar.classList.add('scrolled');
        } else {
            navbar.classList.remove('scrolled');
        }
    });

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('revealed');
            }
        });
    }, { threshold: 0.1 });

    document.querySelectorAll('.scroll-reveal').forEach(el => observer.observe(el));

    document.body.classList.remove('loading');
}

// Chains Filter
function initChainsFilter() {
    const filterButtons = document.querySelectorAll('.chain-filter');

    filterButtons.forEach(button => {
        button.addEventListener('click', () => {
            filterButtons.forEach(btn => btn.classList.remove('active'));
            button.classList.add('active');

            const filter = button.dataset.filter;
            filterChains(filter);
        });
    });

    // Show all chains by default on page load
    filterChains('all');
}

function filterChains(filter) {
    const chainCards = document.querySelectorAll('.chain-card');

    chainCards.forEach(card => {
        const chainType = card.dataset.type;
        let shouldShow = false;

        if (filter === 'all') {
            shouldShow = true;
        } else if (filter === 'substrate') {
            shouldShow = chainType === 'Substrate';
        } else if (filter === 'evm') {
            shouldShow = chainType === 'EVM';
        } else if (filter === 'hybrid') {
            shouldShow = chainType === 'Hybrid';
        }

        if (shouldShow) {
            card.style.display = 'flex';
            card.style.opacity = '0';
            setTimeout(() => {
                card.style.transition = 'opacity 0.3s ease';
                card.style.opacity = '1';
            }, Math.random() * 200);
        } else {
            card.style.transition = 'opacity 0.3s ease';
            card.style.opacity = '0';
            setTimeout(() => {
                card.style.display = 'none';
            }, 300);
        }
    });
}

const chainsData = [
    { name: 'Polkadot', type: 'Substrate', logo: 'polkadot.svg', url: 'https://polkadot.network', description: 'Scalable multi-chain network' },
    { name: 'Ethereum', type: 'EVM', logo: 'ethereum.svg', url: 'https://ethereum.org', description: 'Smart contract platform' },
    { name: 'Kusama', type: 'Substrate', logo: 'kusama.svg', url: 'https://kusama.network', description: 'Polkadot\'s canary network' },
    { name: 'Polygon', type: 'EVM', logo: 'polygon.svg', url: 'https://polygon.technology', description: 'Ethereum scaling solution' },
    { name: 'Moonbeam', type: 'Hybrid', logo: 'moonbeam.svg', url: 'https://moonbeam.network', description: 'Ethereum on Polkadot' },
    { name: 'Binance Smart Chain', type: 'EVM', logo: 'bsc.svg', url: 'https://www.bnbchain.org', description: 'High-performance blockchain' },
    { name: 'Astar', type: 'Hybrid', logo: 'astar.svg', url: 'https://astar.network', description: 'Multi-chain dApp hub' },
    { name: 'Avalanche', type: 'EVM', logo: 'avalanche.svg', url: 'https://www.avax.network', description: 'Fast consensus protocol' },
    { name: 'Acala', type: 'Substrate', logo: 'acala.svg', url: 'https://acala.network', description: 'DeFi hub for Polkadot' },
    { name: 'Arbitrum', type: 'EVM', logo: 'arbitrum.svg', url: 'https://arbitrum.io', description: 'Layer 2 for Ethereum' },
    { name: 'Moonriver', type: 'Hybrid', logo: 'moonriver.svg', url: 'https://moonbeam.network/networks/moonriver', description: 'Ethereum on Kusama' },
    { name: 'Optimism', type: 'EVM', logo: 'optimism.svg', url: 'https://optimism.io', description: 'Optimistic Ethereum' },
    { name: 'Parallel', type: 'Substrate', logo: 'parallel.svg', url: 'https://parallel.fi', description: 'DeFi super app' },
    { name: 'Fantom', type: 'EVM', logo: 'fantom.svg', url: 'https://fantom.foundation', description: 'High-speed consensus' },
    { name: 'Centrifuge', type: 'Substrate', logo: 'centrifuge.svg', url: 'https://centrifuge.io', description: 'Real-world assets on-chain' }
];

function renderChains() {
    const grid = document.getElementById('chains-grid');
    if (!grid) return;

    grid.innerHTML = chainsData.map(chain => `
        <a href="${chain.url}"
           target="_blank"
           rel="noopener"
           class="chain-card scroll-reveal"
           data-type="${chain.type}">
            <div class="chain-logo">
                <img src="assets/logos/${chain.logo}"
                     alt="${chain.name}"
                     loading="lazy"
                     onerror="this.style.display='none'">
            </div>
            <div class="chain-info">
                <div class="chain-name">${chain.name}</div>
                <div class="chain-description">${chain.description}</div>
                <div class="chain-type-badge ${chain.type.toLowerCase()}">${chain.type}</div>
            </div>
            <div class="chain-arrow">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M7 17L17 7M17 7H7M17 7V17"/>
                </svg>
            </div>
        </a>
    `).join('');
}
