import * as pacakgeJson from "./package.json";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  devServer: {
    port: 5173
  },
  devtools: { enabled: true },

  experimental: {
    defaults: {
      nuxtLink: {
        exactActiveClass: 'active',
      }
    }
  },

  routeRules: {
    '/': { prerender: true },
    // These pages are rendered on the client only because they use the Keycloak SDK
    '/dashboard/**': { ssr: false },
  },

  runtimeConfig: {
    public: {
      packageVersion: pacakgeJson.version,
      commitRef: 'unknown',
      keycloak: {
        realm: 'master',
        client: '',
        url: ''
      },
      serverUrl: '',
    }
  },

  modules: [
    "@pinia/nuxt",
    '@bootstrap-vue-next/nuxt',
    "@nuxt/icon",
  ]
})