// @ts-ignore
import { Eta } from "eta"

const eta = new Eta();

export default defineEventHandler(async () => {
    const config = useRuntimeConfig()
    const template = await useStorage('assets:templates').getItemRaw(`firebase-messaging-sw.js.eta`);
    const decodedTemplate = new TextDecoder().decode(template);

    const res = eta.renderString(decodedTemplate, {
        config: config.public
    })
    return new Response(res, {
        headers: {
            "Content-Type": "application/javascript"
        }
    })
})