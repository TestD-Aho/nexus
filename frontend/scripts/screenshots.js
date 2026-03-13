// Screenshot script - Capture all pages
const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

const BASE_URL = process.env.BASE_URL || 'http://localhost:5173';
const OUTPUT_DIR = process.env.OUTPUT_DIR || path.join(__dirname, '..', 'capture');

const pages = [
  { url: '/', name: 'homepage' },
  { url: '/login', name: 'login' },
  { url: '/admin', name: 'admin-redirect' },
  { url: '/page/test', name: 'page-public' },
  { url: '/nonexistent', name: '404' },
];

async function takeScreenshot() {
  // Ensure output directory exists
  if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  }

  console.log(`📸 Starting screenshot capture...`);
  console.log(`Base URL: ${BASE_URL}`);
  console.log(`Output: ${OUTPUT_DIR}\n`);

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1920, height: 1080 }
  });
  const page = await context.newPage();

  const results = [];

  for (const p of pages) {
    const url = `${BASE_URL}${p.url}`;
    const filename = `${p.name}.png`;
    const filepath = path.join(OUTPUT_DIR, filename);

    try {
      console.log(`📷 Capturing: ${p.url}...`);
      
      // Handle console errors
      const errors = [];
      page.on('console', msg => {
        if (msg.type() === 'error') errors.push(msg.text());
      });

      await page.goto(url, { waitUntil: 'networkidle', timeout: 30000 });
      
      // Wait a bit for any dynamic content
      await page.waitForTimeout(1000);

      // Take screenshot
      await page.screenshot({ 
        path: filepath, 
        fullPage: false,
        animations: 'disabled'
      });

      const fileSize = fs.statSync(filepath).size;
      console.log(`   ✅ Saved: ${filename} (${(fileSize / 1024).toFixed(1)} KB)`);
      
      results.push({ page: p.name, status: 'success', file: filename });
      
    } catch (error) {
      console.log(`   ❌ Failed: ${p.url} - ${error.message}`);
      results.push({ page: p.name, status: 'error', error: error.message });
    }
  }

  await browser.close();

  // Summary
  console.log('\n📊 Summary:');
  const success = results.filter(r => r.status === 'success').length;
  const failed = results.filter(r => r.status === 'error').length;
  console.log(`   ✅ Success: ${success}`);
  console.log(`   ❌ Failed: ${failed}`);
  
  if (failed > 0) {
    console.log('\n⚠️ Failed pages:');
    results.filter(r => r.status === 'error').forEach(r => {
      console.log(`   - ${r.page}: ${r.error}`);
    });
  }

  console.log(`\n📁 Screenshots saved to: ${OUTPUT_DIR}`);
  
  return results;
}

// Run if called directly
if (require.main === module) {
  takeScreenshot()
    .then(() => process.exit(0))
    .catch(err => {
      console.error('Fatal error:', err);
      process.exit(1);
    });
}

module.exports = { takeScreenshot };
