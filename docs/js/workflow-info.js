/**
 * Workflow Simulator Information
 * Explains the purpose and functionality of the workflow simulator
 */

const WORKFLOW_INFO = {
    title: 'Transaction Lifecycle Visualization',
    description: 'The workflow simulator demonstrates the complete transaction lifecycle in Apex SDK Protocol, showing how cross-chain transactions flow through our unified system architecture.',
    steps: [
        {
            name: 'Initialize SDK',
            description: 'SDK connects to configured blockchain endpoints (Substrate and/or EVM)',
            color: '#3b82f6'
        },
        {
            name: 'Build Transaction',
            description: 'Transaction builder creates a unified transaction object with type safety',
            color: '#8b5cf6'
        },
        {
            name: 'Sign Transaction',
            description: 'Transaction is signed using the appropriate cryptographic scheme for the target chain',
            color: '#06b6d4'
        },
        {
            name: 'Submit to Network',
            description: 'Transaction is submitted to the target blockchain network via RPC',
            color: '#10b981'
        },
        {
            name: 'Confirm Block',
            description: 'Transaction is confirmed when included in a finalized block',
            color: '#f59e0b'
        }
    ],
    purpose: 'This visualization helps developers understand the seamless flow of cross-chain transactions, demonstrating how Apex SDK Protocol abstracts away the complexity of different blockchain protocols into a single, unified interface.'
};

if (typeof window !== 'undefined') {
    window.WORKFLOW_INFO = WORKFLOW_INFO;
}
