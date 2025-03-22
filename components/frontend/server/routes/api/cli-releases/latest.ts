import { listReleases } from ".";

export default defineEventHandler(async event => {
    const releases = await listReleases(event);
    if (!releases.releases.length) {
        throw createError({
            statusCode: 404,
            statusMessage: "No releases found",
        });
    }
    return releases.releases[0];
});