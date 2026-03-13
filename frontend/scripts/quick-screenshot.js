// Quick screenshot script
const { chromium } = require('playwright');

async function main() {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1920, height: 1080 } });
  
  const pages = [
    { url: 'http://localhost:5173/', name: 'homepage' },
    { url: 'http://localhost:5173/login', name: 'login' },
    { url: 'http://localhost:5173/page/test', name: 'page-public' },
    { url: 'http://localhost:5173/nonexistent', name: '404' },
  ];

  for (const p of pages) {
    try {
      await page.goto(p.url, { waitUntil: 'networkidle', timeout: 15000 });
      await page.waitForTimeout(500);
      await page.screenshot({ path: `../capture/${p.name}.png`, fullPage: false });
      console.log(`✅ ${p.name}.png`);
    } catch (e) {
      console.log(`❌ ${p.name}: ${e.message}`);
    }
  }

  await browser.close();
}

main();
