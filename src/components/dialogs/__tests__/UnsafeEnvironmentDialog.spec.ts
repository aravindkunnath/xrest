import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import UnsafeEnvironmentDialog from '../UnsafeEnvironmentDialog.vue'

describe('UnsafeEnvironmentDialog', () => {
    beforeEach(() => {
        vi.clearAllMocks()
    })

    const mountDialog = (props = { open: true, environmentName: 'PROD', countdown: 10 }) => {
        return mount(UnsafeEnvironmentDialog, {
            props,
            global: {
                stubs: {
                    DialogPortal: {
                        template: '<div><slot /></div>'
                    },
                    transition: false
                }
            }
        })
    }

    it('renders when open is true', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PROD', countdown: 10 })
        await wrapper.vm.$nextTick()

        expect(wrapper.text()).toContain('Unsafe Environment!')
        expect(wrapper.text()).toContain('PROD')
    })

    it('does not render when open is false', async () => {
        const wrapper = mountDialog({ open: false, environmentName: 'PROD', countdown: 10 })
        await wrapper.vm.$nextTick()

        // Dialog content should not be visible
        const dialogContent = wrapper.find('[class*="DialogContent"]')
        expect(dialogContent.exists()).toBe(false)
    })

    it('displays the environment name correctly', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PRODUCTION', countdown: 10 })
        await wrapper.vm.$nextTick()

        expect(wrapper.text()).toContain('PRODUCTION')
    })

    it('displays the countdown timer', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PROD', countdown: 7 })
        await wrapper.vm.$nextTick()

        expect(wrapper.text()).toContain('7s')
    })

    it('updates countdown display when prop changes', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PROD', countdown: 10 })
        await wrapper.vm.$nextTick()

        expect(wrapper.text()).toContain('10s')

        await wrapper.setProps({ countdown: 5 })
        expect(wrapper.text()).toContain('5s')

        await wrapper.setProps({ countdown: 1 })
        expect(wrapper.text()).toContain('1s')
    })

    it('displays progress bar that reflects countdown', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PROD', countdown: 5 })
        await wrapper.vm.$nextTick()

        // Find the progress bar element
        const progressBar = wrapper.find('[class*="bg-destructive"][class*="transition"]')
        expect(progressBar.exists()).toBe(true)

        // At countdown 5, width should be 50% (5/10 * 100)
        expect(progressBar.attributes('style')).toContain('50%')

        // Update countdown to 2
        await wrapper.setProps({ countdown: 2 })
        expect(progressBar.attributes('style')).toContain('20%')
    })

    it('emits proceed event when "I Accept the Risk" button is clicked', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PROD', countdown: 10 })
        await wrapper.vm.$nextTick()

        const buttons = wrapper.findAll('button')
        const proceedButton = buttons.find(b => b.text().includes('I Accept the Risk'))

        expect(proceedButton).toBeDefined()
        await proceedButton?.trigger('click')

        expect(wrapper.emitted('proceed')).toBeDefined()
        expect(wrapper.emitted('proceed')?.length).toBe(1)
    })

    it('emits cancel event when "Cancel" button is clicked', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PROD', countdown: 10 })
        await wrapper.vm.$nextTick()

        const buttons = wrapper.findAll('button')
        const cancelButton = buttons.find(b => b.text().includes('Cancel'))

        expect(cancelButton).toBeDefined()
        await cancelButton?.trigger('click')

        expect(wrapper.emitted('cancel')).toBeDefined()
        expect(wrapper.emitted('cancel')?.length).toBe(1)
    })

    it('shows warning styling for unsafe environment', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PROD', countdown: 10 })
        await wrapper.vm.$nextTick()

        // Check for destructive/warning colors
        const dialogContent = wrapper.find('[class*="border-destructive"]')
        expect(dialogContent.exists()).toBe(true)

        const alertIcon = wrapper.find('[class*="text-destructive"][class*="animate-pulse"]')
        expect(alertIcon.exists()).toBe(true)
    })

    it('displays correct warning message', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'STAGING', countdown: 10 })
        await wrapper.vm.$nextTick()

        expect(wrapper.text()).toContain('You are about to execute a request against')
        expect(wrapper.text()).toContain('STAGING')
        expect(wrapper.text()).toContain('This environment is marked as unsafe')
    })

    it('shows confirmation question', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PROD', countdown: 10 })
        await wrapper.vm.$nextTick()

        expect(wrapper.text()).toContain('Are you sure you want to proceed with this action?')
    })

    it('shows auto-cancel warning', async () => {
        const wrapper = mountDialog({ open: true, environmentName: 'PROD', countdown: 10 })
        await wrapper.vm.$nextTick()

        expect(wrapper.text()).toContain('Request will be cancelled in')
    })
})
