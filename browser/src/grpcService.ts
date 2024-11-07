import { DeepPartial, HttpMonitorExecutorServiceImplementation, HttpRequest, HttpResponse } from "../compiled_proto/http-monitor-executor.js";
import { BrowserPool } from "./browser.js";

export function httpMonitorExecutorImpl(browserPool: BrowserPool): HttpMonitorExecutorServiceImplementation {
    return {
        executeHttpRequest: async (request: HttpRequest): Promise<DeepPartial<HttpResponse>> => {
            const browser = await browserPool.getBrowser();
            const response = await browser.fetchPage(request);
            return response;
        }
    };
};
