import { createRouter, createWebHistory } from 'vue-router'
import ServicesView from '../views/ServicesView.vue'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
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
            path: '/collections',
            name: 'collections',
            component: () => import('../views/CollectionsView.vue')
        },
        {
            path: '/history',
            name: 'history',
            component: () => import('../views/HistoryView.vue')
        },
        {
            path: '/secrets',
            name: 'secrets',
            component: () => import('../views/SecretsView.vue')
        },
        {
            path: '/workflow',
            name: 'workflow',
            component: () => import('../views/WorkflowView.vue')
        },
        {
            path: '/settings',
            name: 'settings',
            component: () => import('../views/SettingsView.vue')
        }
    ]
})

export default router
