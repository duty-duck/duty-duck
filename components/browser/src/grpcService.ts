import { DeepPartial, BrowserServiceImplementation, HttpRequest, HttpResponse } from "../compiled_proto/browser.js";
import { BrowserPool } from "./browser.js";

export function browserServiceImpl(browserPool: BrowserPool): BrowserServiceImplementation {
    return {
        executeHttpRequest: async (request: HttpRequest): Promise<DeepPartial<HttpResponse>> => {
            const browser = await browserPool.getBrowser();
            const response = await browser.fetchPage(request);
            return response;
        }
    };
};
