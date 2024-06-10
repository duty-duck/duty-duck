import { createRouter, createWebHistory } from 'vue-router'
import DashboardLayout from '@/layouts/DashboardLayout.vue'
import PublicLayout from '@/layouts/PublicLayout.vue'

// Views
import HomeView from '@/views/public/HomeView.vue'
import DashboardHomeView from '@/views/dashboard/HomeView.vue'
import MonitorsView from '@/views/dashboard/MonitorsView.vue'
import SignupView from '@/views/public/SignupView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  linkExactActiveClass: 'active',
  routes: [
    {
      path: '/',
      component: PublicLayout,
      children: [
        {

          path: '',
          component: HomeView,
        },
        {
          path: 'signup',
          component: SignupView
        }
      ]
    },
    {
      path: '/dashboard',
      component: DashboardLayout,
      children: [
        {
          path: '',
          name: 'dashboard',
          components: {
            default: DashboardHomeView
          }
        },
        {
          path: 'monitors',
          name: 'monitors',
          components: {
            default: MonitorsView
          }
        }
      ]
    }
  ]
})

export default router
