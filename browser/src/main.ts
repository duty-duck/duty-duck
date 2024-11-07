import { HttpMonitorExecutorDefinition } from "../compiled_proto/http-monitor-executor.js";
import { createServer } from "nice-grpc";
import { httpMonitorExecutorImpl } from "./grpcService.js";
import { createLogger } from "./logger.js";
import { createBrowserPool } from "./browser.js";

const log = createLogger("main");

const config = {
    grpcPort: process.env.GRPC_PORT ? parseInt(process.env.GRPC_PORT) : 50051,
    maxConcurrentBrowsers: process.env.MAX_CONCURRENT_BROWSERS ? parseInt(process.env.MAX_CONCURRENT_BROWSERS) : 10,
    maxConcurrentPagesPerBrowser: process.env.MAX_CONCURRENT_PAGES_PER_BROWSER ? parseInt(process.env.MAX_CONCURRENT_PAGES_PER_BROWSER) : 40,
}
log.info("Initializing browser pool");
const browserPool = await createBrowserPool(config.maxConcurrentBrowsers, {
    maxOpenPages: config.maxConcurrentPagesPerBrowser,
});

log.info({ config }, "Starting gRPC Server");
const server = createServer();
server.add(HttpMonitorExecutorDefinition, httpMonitorExecutorImpl(browserPool));

await server.listen(`0.0.0.0:${config.grpcPort}`);
log.info(`gRPC Server is running on port ${config.grpcPort}`);
