export default defineEventHandler(async event => {
    const config = useRuntimeConfig();
    const query: { url: string } = getQuery(event);
    const response = await fetch(query.url);
    const text = await response.text();
    const replacedText = text.replace("const config = {};", `const config = ${JSON.stringify(config.public)};`);
    return new Response(replacedText, { headers: response.headers })
})
