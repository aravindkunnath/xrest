import { describe, it, expect, beforeEach, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createRouter, createWebHistory } from 'vue-router'
import VSCodeSidebar from '@/components/VSCodeSidebar.vue'
import { useI18n } from '@/composables/useI18n'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/services', component: { template: '<div>Services</div>' } },
    { path: '/environments', component: { template: '<div>Environments</div>' } },
    { path: '/settings', component: { template: '<div>Settings</div>' } }
  ]
})

describe('VSCodeSidebar Behavioral & Parameterized Routing Tests', () => {
  beforeEach(async () => {
    router.push('/services')
    await router.isReady()
  })

  // Parameterized Test on routing
  const routesToTest = [
    { path: '/services' },
    { path: '/environments' },
    { path: '/settings' }
  ]

  it.each(routesToTest)('navigates to %s on click', async ({ path }) => {
    const pushSpy = vi.spyOn(router, 'push')
    const wrapper = mount(VSCodeSidebar, {
      global: {
        plugins: [router],
        stubs: {
          TooltipProvider: { template: '<div><slot /></div>' },
          Tooltip: { template: '<div><slot /></div>' },
          TooltipTrigger: { template: '<div><slot /></div>' },
          TooltipContent: { template: '<div><slot /></div>' }
        }
      }
    })

    const link = wrapper.find(`[href="${path}"]`)
    expect(link.exists()).toBe(true)
    await link.trigger('click')
    expect(pushSpy).toHaveBeenCalledWith(path)
    pushSpy.mockRestore()
  })

  // Parameterized Test on tooltip internationalization translation
  const i18nScenarios = [
    { locale: 'en', services: 'Services', environments: 'Environments', settings: 'Settings' },
    { locale: 'fr', services: 'Services', environments: 'Environnements', settings: 'Paramètres' }
  ]

  it.each(i18nScenarios)('updates tooltips correctly for locale: %s', async ({ locale, services, environments, settings }) => {
    const wrapper = mount(VSCodeSidebar, {
      global: {
        plugins: [router],
        stubs: {
          TooltipProvider: { template: '<div><slot /></div>' },
          Tooltip: { template: '<div><slot /></div>' },
          TooltipTrigger: { template: '<div><slot /></div>' },
          TooltipContent: { template: '<div class="tooltip-content"><slot /></div>' }
        }
      }
    })

    const i18n = useI18n()
    i18n.locale.value = locale
    await wrapper.vm.$nextTick()

    const contents = wrapper.findAll('.tooltip-content')
    expect(contents.at(0)?.text()).toBe(services)
    expect(contents.at(1)?.text()).toBe(environments)
    expect(contents.at(2)?.text()).toBe(settings)
  })
})
