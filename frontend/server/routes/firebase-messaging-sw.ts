// @ts-ignore
import { Eta } from "eta"

const eta = new Eta();

export default defineEventHandler(async () => {
    const config = useRuntimeConfig()
    const template = await useStorage('assets:templates').getItem(`firebase-messaging-sw.js.eta`);
    const res = eta.renderString(template, {
        config: config.public
    })
    return new Response(res, {
        headers: {
            "Content-Type": "application/javascript"
        }
    })
})