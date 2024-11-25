import { protoCamelCase } from "@bufbuild/protobuf/reflect";
import { createBrowserPool } from "./browser.js";

test('it should download this page', async () => {
    const pool = await createBrowserPool(1, { maxOpenPages: 1 });
    const browser = await pool.getBrowser();

    await browser.fetchPage({ endpoint: 'https://www.google.com', requestTimeoutMs: 10000, httpHeaders: {} });
    await pool.close();
});
