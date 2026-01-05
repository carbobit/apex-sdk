/**
 * Supported Chains Data
 * Comprehensive list of all supported blockchain networks
 */

const SUPPORTED_CHAINS = {
    substrate: [
        { name: 'Polkadot', status: 'stable', type: 'Relay Chain', logo: 'assets/logos/polkadot.svg', color: '#000000', website: 'https://polkadot.network' },
        { name: 'Kusama', status: 'stable', type: 'Relay Chain', logo: 'assets/logos/kusama.svg', color: '#000000', website: 'https://kusama.network' },
        { name: 'Paseo', status: 'testnet', type: 'Testnet', logo: 'assets/logos/polkadot.svg', color: '#000000', website: 'https://polkadot.network' },
        { name: 'Westend', status: 'testnet', type: 'Testnet', logo: 'assets/logos/polkadot.svg', color: '#000000', website: 'https://polkadot.network' }
    ],
    evm: [
        { name: 'Ethereum', status: 'stable', type: 'Mainnet', logo: 'assets/logos/ethereum.svg', color: '#627EEA', website: 'https://ethereum.org' },
        { name: 'Binance Smart Chain', status: 'stable', type: 'Mainnet', logo: 'assets/logos/bsc.svg', color: '#F0B90B', website: 'https://www.bnbchain.org' },
        { name: 'Polygon', status: 'stable', type: 'Mainnet', logo: 'assets/logos/polygon.svg', color: '#8247E5', website: 'https://polygon.technology' },
        { name: 'Avalanche', status: 'stable', type: 'Mainnet', logo: 'assets/logos/avalanche.svg', color: '#E84142', website: 'https://www.avax.network' }
    ],
    hybrid: [
        { name: 'Moonbeam', status: 'stable', type: 'Parachain', logo: 'assets/logos/moonbeam.svg', color: '#53CBC8', website: 'https://moonbeam.network' },
        { name: 'Moonriver', status: 'stable', type: 'Parachain', logo: 'assets/logos/moonriver.svg', color: '#53CBC8', website: 'https://moonbeam.network/networks/moonriver' },
        { name: 'Astar', status: 'stable', type: 'Parachain', logo: 'assets/logos/astar.svg', color: '#1A73E8', website: 'https://astar.network' }
    ]
};

// Export for use in other modules
if (typeof window !== 'undefined') {
    window.SUPPORTED_CHAINS = SUPPORTED_CHAINS;
}
