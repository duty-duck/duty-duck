import { Sema } from "async-sema";
import puppeteer, { Page, PuppeteerError, TimeoutError } from "puppeteer-core";
import { createLogger } from "./logger.js";
import { HttpErrorKind, HttpRequest, HttpResponse } from "../compiled_proto/browser.js";
import { Resolver } from "node:dns/promises";

type BrowserOptions = {
    maxOpenPages: number;
}

type Browser = {
    fetchPage: (request: HttpRequest) => Promise<HttpResponse>;
}

const logger = createLogger("browser");

const ChromeErrorStatusMapping: Record<string, [HttpErrorKind, string]> = {
    ERR_NAME_NOT_RESOLVED: [HttpErrorKind.CONNECT, "The name did not resolve"],
    ERR_CONNECTION_TIMED_OUT: [HttpErrorKind.TIMEOUT, "The connection timed out"],
    ERR_TIMED_OUT: [HttpErrorKind.TIMEOUT, "The operation timed out"],
    ERR_CONNECTION_REFUSED: [HttpErrorKind.CONNECT, "The connection was refused"],
    ERR_CONNECTION_RESET: [HttpErrorKind.CONNECT, "The connection was reset"],
    ERR_ABORTED: [HttpErrorKind.CONNECT, "The operation was aborted"],
    ERR_CONTENT_DECODING_FAILED: [HttpErrorKind.DECODE, "The content decoding failed"],
    ERR_TOO_MANY_REDIRECTS: [HttpErrorKind.REDIRECT, "Too many redirects"],
};

const createBrowser = async (options: BrowserOptions): Promise<Browser> => {
    const semaphore = new Sema(options.maxOpenPages);
    const resolver = new Resolver();

    const browser = await puppeteer.launch({
        headless: true,
        executablePath: "/usr/bin/chromium",
        args: ["--no-sandbox", "--disable-setuid-sandbox"],
    });

    const acquirePermit = async () => {
        let t0 = setTimeout(() => {
            logger.warn("A process has been blocked for 30 seconds while waiting for a browser permit. Increase the maxOpenPages option to allow more concurrent pages.");
        }, 30000);

        let t1 = setTimeout(() => {
            logger.warn("A process has been blocked for 60 seconds while waiting for a browser permit. Increase the maxOpenPages option to allow more concurrent pages.");
        }, 60000);

        await semaphore.acquire();
        clearTimeout(t0);
        clearTimeout(t1);
    }

    const releasePermit = () => {
        semaphore.release();
    }

    const fetchPage = async (request: HttpRequest, abortSignal?: AbortSignal): Promise<HttpResponse> => {
        await acquirePermit();

        let page: Page | undefined;
        // create an empty response
        const response: HttpResponse = {
            httpCode: undefined,
            screenshot: undefined,
            httpHeaders: {},
            responseTimeMs: 0,
            responseIpAddress: undefined,
            resolvedIpAddresses: [],
            responseBodySizeBytes: 0,
            responseBodyContent: undefined,
            error: undefined,
            errorMessage: undefined,
        }

        try {
            // open a new page
            page = await browser.newPage();

            // configure the page
            page.setDefaultTimeout(request.requestTimeoutMs);
            await page.setCacheEnabled(false);
            await page.setViewport({ width: 1280, height: 800 });
            await page.setExtraHTTPHeaders(request.httpHeaders);

            // navigate to the endpoint and wait for the page to load
            // measure the time it takes to load the page
            const fetchStart = performance.now();
            const pageResponse = await page.goto(request.endpoint, { waitUntil: "load", timeout: request.requestTimeoutMs, signal: abortSignal });
            response.responseTimeMs = Math.round(performance.now() - fetchStart);

            // get the response headers
            response.httpHeaders = pageResponse.headers();

            // get the response IP address
            response.responseIpAddress = pageResponse.remoteAddress().ip;

            // get the http code
            response.httpCode = pageResponse?.status();
            if (response.httpCode >= 400) {
                response.error = HttpErrorKind.HTTP_CODE;
                response.errorMessage = `Invalid HTTP code ${response.httpCode}`;
            }

            // get the response body
            response.responseBodyContent = new TextEncoder().encode(await page.content());
            response.responseBodySizeBytes = response.responseBodyContent.length;

            // take a screenshot
            const screenshot = await page.screenshot({ type: "jpeg", quality: 80 });
            response.screenshot = {
                data: screenshot,
                contentType: "image/jpeg",
            };

            // resolve all the IP addresses of this endpoint
            const domain = new URL(request.endpoint).hostname;
            try {
                response.resolvedIpAddresses = await resolver.resolve(domain)
            } catch (error) {
                logger.debug({ error }, "An error occurred while resolving addresses");
                response.resolvedIpAddresses = [response.responseIpAddress];
            }
        }
        // Catch all errors and map them to an error kind and a message
        catch (error) {
            if (error instanceof TimeoutError) {
                response.error = HttpErrorKind.TIMEOUT;
                response.errorMessage = "The page took too long to load";
                logger.debug({ error }, "A timeout error occurred while fetching a page");
            } else {
                for (const [key, [errorKind, errorMessage]] of Object.entries(ChromeErrorStatusMapping)) {
                    if (error.message.includes(key)) {
                        logger.debug({ error }, "A Chrome error occurred while fetching a page");

                        response.error = errorKind;
                        response.errorMessage = errorMessage;
                        break;
                    }
                }

                if (!response.error) {
                    response.error = HttpErrorKind.UNKNOWN;
                    response.errorMessage = "An unknown error occurred";
                    const errorObject = error instanceof Error ? { errorMessage: error.message, errorCause: error.cause, errorStack: error.stack } : error;
                    logger.error({ endpoint: request.endpoint, ...errorObject }, "An unknown error occurred while fetching a page");
                }
            }
        }
        // Always close the page and release the semaphore permit
        finally {
            if (page) {
                await page.close();
            }
            releasePermit();
        }

        return response;
    }

    return { fetchPage };
}

export type BrowserPool = Awaited<ReturnType<typeof createBrowserPool>>;

/**
 * Create a pool of browsers.
 * @param numBrowsers - The number of browsers to create.
 * @param options - The options for the browsers.
 * @returns A browser pool.
 */
export const createBrowserPool = async (numBrowsers: number, options: BrowserOptions) => {
    logger.info({ numBrowsers, options }, "Creating browser pool");

    const browsers = await Promise.all(Array.from({ length: numBrowsers }, () => createBrowser(options)));
    let nextBrowserIndex = 0;
    return {
        /**
         * Get a browser from the pool.
         * Browsers are returned in a round-robin fashion.
         * @returns A browser instance.
         */
        getBrowser: async () => {
            const browser = browsers[nextBrowserIndex];
            nextBrowserIndex = (nextBrowserIndex + 1) % numBrowsers;
            return browser;
        }
    }
}