#!/usr/bin/env node

/**
 * Cloudflare Pages Build Script
 * Injects WEB3FORMS_ACCESS_KEY from Cloudflare environment variables
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const DIST_DIR = path.join(__dirname, '..', 'dist');
const DOCS_DIR = path.join(__dirname, '..', 'docs');

// Get the Web3Forms access key from environment
const WEB3FORMS_KEY = process.env.WEB3FORMS_ACCESS_KEY;

console.log('🚀 Starting Cloudflare Pages build...');

// Step 1: Create dist directory
console.log('📁 Creating dist directory...');
if (!fs.existsSync(DIST_DIR)) {
  fs.mkdirSync(DIST_DIR, { recursive: true });
}

// Step 2: Copy docs to dist
console.log('📋 Copying docs to dist...');
try {
  execSync(`cp -r ${DOCS_DIR}/* ${DIST_DIR}/`, { stdio: 'inherit' });
  console.log('✓ Docs copied successfully');
} catch (error) {
  console.error('✗ Error copying docs:', error.message);
  process.exit(1);
}

// Step 3: Inject Web3Forms key into config.js
const configPath = path.join(DIST_DIR, 'js', 'config.js');

if (!fs.existsSync(configPath)) {
  console.error('✗ Error: config.js not found at', configPath);
  process.exit(1);
}

console.log('🔑 Injecting Web3Forms access key...');

if (!WEB3FORMS_KEY) {
  console.warn('⚠ Warning: WEB3FORMS_ACCESS_KEY environment variable not set!');
  console.warn('  The contact form will not work without this key.');
  console.warn('  Set it in Cloudflare Pages: Settings > Environment variables');
} else {
  try {
    let configContent = fs.readFileSync(configPath, 'utf8');

    // Replace the placeholder with the actual key
    const originalContent = configContent;
    configContent = configContent.replace(
      /__WEB3FORMS_ACCESS_KEY__/g,
      WEB3FORMS_KEY
    );

    // Verify replacement happened
    if (configContent === originalContent) {
      console.warn('⚠ Warning: No placeholder found in config.js to replace');
    } else if (configContent.includes('__WEB3FORMS_ACCESS_KEY__')) {
      console.error('✗ Error: Some placeholders were not replaced!');
      process.exit(1);
    } else {
      fs.writeFileSync(configPath, configContent, 'utf8');
      console.log('✓ Web3Forms access key injected successfully');
    }
  } catch (error) {
    console.error('✗ Error injecting access key:', error.message);
    process.exit(1);
  }
}

// Step 4: Verify the build
console.log('🔍 Verifying build...');
const verifyFiles = [
  'index.html',
  'js/config.js',
  'css/main.css'
];

let allFilesExist = true;
for (const file of verifyFiles) {
  const filePath = path.join(DIST_DIR, file);
  if (fs.existsSync(filePath)) {
    console.log(`  ✓ ${file}`);
  } else {
    console.error(`  ✗ Missing: ${file}`);
    allFilesExist = false;
  }
}

if (!allFilesExist) {
  console.error('✗ Build verification failed!');
  process.exit(1);
}

console.log('✅ Build completed successfully!');
console.log(`📦 Output directory: ${DIST_DIR}`);
