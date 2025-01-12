import { S3Client, ListObjectsCommand, CommonPrefix } from "@aws-sdk/client-s3";

export type Release = {
    name: string,
    platforms: { storageKey: string, platform: string, version: string, fileName: string, url: string }[]
}

export type Response = {
    releases: Release[]
}

const buildRelease = async (s3CommonPrefix: CommonPrefix, s3Client: S3Client, awsRegion: string): Promise<Release> => {
    // build the release name from the prefix, removing the "cli/" prefix and the trailing "/"
    const releaseName = s3CommonPrefix.Prefix?.replace("cli/", "")!.replace("/", "");

    const releasePlatformsResponse = await s3Client.send(new ListObjectsCommand({
        Bucket: "dutyduck-releases",
        Prefix: s3CommonPrefix.Prefix,
        Delimiter: "/"
    }));
    
    const releasePlatforms = releasePlatformsResponse.Contents?.map((object) => {
        const [_, version, fileName] = object.Key!.split("/");
        const [os, arch] = fileName.split("-");
        return { 
            fileName,
            version,
            storageKey: object.Key!,
            platform: `${os}-${arch}`,
            url: `https://dutyduck-releases.s3.${awsRegion}.amazonaws.com/${object.Key!}`
        }
    })

    return {
        name: releaseName!,
        platforms: releasePlatforms ?? []
    }
}

// A poor man's cache to save S3 calls, but it's good enough for now
const cacheDuration = 4 * 3600 * 1000; // 4 hours
const lastResponseCache: { response: Response | null, builtAt: Date | null } = {
    response: null,
    builtAt: null
}

export default defineEventHandler(async (event) => {
    // Return the cached response if it's still valid
    if (lastResponseCache.response && lastResponseCache.builtAt && lastResponseCache.builtAt.getTime() + cacheDuration > Date.now()) {
        return lastResponseCache.response;
    }

    const { awsAccessKeyId, awsSecretAccessKey, awsRegion } = useRuntimeConfig(event);

    const client = new S3Client({
        region: awsRegion,
        credentials: {
            accessKeyId: awsAccessKeyId,
            secretAccessKey: awsSecretAccessKey
        }
    });

    const listReleasesCommand = new ListObjectsCommand({
        Bucket: "dutyduck-releases",
        Prefix: "cli/",
        Delimiter: "/"
    });
    const releasesFolders = (await client.send(listReleasesCommand)).CommonPrefixes ?? [];
    const releases = await Promise.all(releasesFolders.map((folder) => buildRelease(folder, client, awsRegion)));

    const response: Response = {
        releases
    }

    lastResponseCache.response = response;
    lastResponseCache.builtAt = new Date();

    console.log(`Fetched ${releases.length} CLI releases from S3`);
    return response;

})