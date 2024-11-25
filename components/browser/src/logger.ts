import { pino } from "pino";

const logLevel = process.env.LOG_LEVEL || "info";
const logger = pino({
    level: logLevel,
});

export const createLogger = (module: string) => logger.child({ module });
