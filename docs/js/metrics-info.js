/**
 * Metrics Information
 * Explains the source and purpose of real-time metrics
 */

const METRICS_INFO = {
    sources: {
        tps: {
            label: 'Transactions per Second',
            source: 'Aggregated from connected blockchain networks',
            note: 'Currently simulated for demonstration. In production, connects to live RPC endpoints.',
            updateInterval: 2000
        },
        blocks: {
            label: 'Blocks Processed',
            source: 'Cumulative block count across all monitored chains',
            note: 'Simulated data. Production version tracks actual block production.',
            updateInterval: 3000
        },
        chains: {
            label: 'Supported Chains',
            source: 'Static count of all supported blockchain networks',
            note: 'Includes Substrate, EVM, and hybrid chains',
            updateInterval: null
        },
        uptime: {
            label: 'System Uptime',
            source: 'SDK connection health monitoring',
            note: 'Based on successful RPC connections and transaction success rates',
            updateInterval: null
        }
    },
    production: {
        enabled: false,
        endpoints: {
            substrate: null, // To be configured
            evm: null // To be configured
        },
        note: 'Real-time metrics require connection to blockchain RPC endpoints. Configure in production environment.'
    }
};

if (typeof window !== 'undefined') {
    window.METRICS_INFO = METRICS_INFO;
}
