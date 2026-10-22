import { createRouter, createWebHashHistory } from 'vue-router'
import ServicesView from "@/features/services/views/ServicesView.vue"

const router = createRouter({
    history: createWebHashHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: '/',
            redirect: '/services'
        },
        {
            path: '/services',
            name: 'services',
            component: ServicesView
        },
        {
            path: '/environments',
            name: 'environments',
            component: () => import("@/features/environments/views/EnvironmentsView.vue")
        },
        {
            path: '/collections',
            name: 'collections',
            component: () => import("@/features/collections/views/CollectionsView.vue")
        },
        {
            path: '/history',
            name: 'history',
            component: () => import("@/features/history/views/HistoryView.vue")
        },
        {
            path: '/secrets',
            name: 'secrets',
            component: () => import("@/features/secrets/views/SecretsView.vue")
        },
        {
            path: '/workflow',
            name: 'workflow',
            component: () => import("@/features/workflows/views/WorkflowView.vue")
        },
        {
            path: '/settings',
            name: 'settings',
            component: () => import("@/features/settings/views/SettingsView.vue")
        }
    ]
})

export default router

