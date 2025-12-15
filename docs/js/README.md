# JavaScript Modules Documentation

This directory contains the modular JavaScript architecture for the Apex SDK documentation site.

## Module Structure

### Core Modules

#### `config.js`
Centralized configuration for the entire application.
- SEO settings
- API endpoints
- Animation preferences
- Feature flags
- Personalization settings

**Usage:**
```javascript
// Access via window.CONFIG
const featureEnabled = window.CONFIG.features.codeEditor;
```

#### `seo.js`
SEO management and optimization.
- Meta tag management
- Structured data (JSON-LD)
- Open Graph tags
- Twitter Cards
- Apex-specific SEO optimization (differentiates from other "Apex" products)

**Features:**
- Automatically optimizes for "Apex SDK Protocol" to avoid confusion
- Adds alternate names for better search visibility
- Updates meta tags dynamically

#### `main.js`
Main application orchestrator.
- Initializes all modules
- Sets up animations
- Handles navigation
- Coordinates feature initialization

### Feature Modules

#### `particles.js`
Particle system for background effects.
- Canvas-based particle rendering
- Connection algorithms
- Performance optimized

#### `blockchain-viz.js`
3D blockchain network visualization using Three.js.
- WebGL rendering
- Dynamic camera movements
- Network node animations

#### `metrics.js`
Real-time blockchain metrics display.
- Animated counters
- Smooth number transitions
- Simulated live data (ready for API integration)

#### `workflow-simulator.js`
Animated workflow visualization.
- Protocol flow simulation
- Step-by-step animations
- Interactive controls

#### `personalization.js`
Adaptive content personalization engine.
- User behavior tracking
- Expertise level detection
- **Markdown content rendering** in recommendations
- Personalized content paths

## Module Loading Order

1. `config.js` - Must load first (sets window.CONFIG)
2. `seo.js` - Initializes SEO immediately
3. Feature modules (particles, blockchain-viz, etc.)
4. `personalization.js` - Uses other modules
5. `main.js` - Orchestrates everything

## Personalization Features

The personalization module now:
- **Renders actual markdown content** in recommendation cards
- Loads markdown files and extracts previews
- Shows clickable cards with descriptions
- Links directly to viewer.html with the document

## SEO Optimization

The SEO module specifically addresses the "Apex" naming conflict by:
- Using "Apex SDK Protocol" as the primary name
- Adding alternate names: "Apex SDK blockchain", "Apex SDK Rust", "Apex SDK Substrate"
- Including specific keywords to differentiate from other Apex products
- Enhanced structured data with brand information

## Adding New Modules

1. Create the module file in `js/`
2. Access `window.CONFIG` for configuration
3. Initialize in `main.js` if needed
4. Add to HTML in the correct load order

## Browser Compatibility

- Modern browsers (Chrome, Firefox, Safari, Edge)
- ES6+ features used
- Fallbacks for older browsers where needed
- Progressive enhancement approach
