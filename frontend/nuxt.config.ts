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
    '/signup': { ssr: false },
    '/dashboard/**': { ssr: false },
  },

  runtimeConfig: {
    public: {
      keycloak: {
        realm: 'master',
        client: '',
        url: ''
      },
      serverUrl: '',
    }
  },

  modules: ["@pinia/nuxt"]
})