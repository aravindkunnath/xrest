import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import ImportCurlDialog from '../ImportCurlDialog.vue'
import { createTestingPinia } from '@pinia/testing'
import { useServicesStore } from '@/stores/services'
import { toast } from 'vue-sonner'

// Mock vue-sonner
vi.mock('vue-sonner', () => ({
    toast: {
        error: vi.fn(),
        success: vi.fn(),
    },
}))

describe('ImportCurlDialog', () => {
    let pinia: any
    let servicesStore: any

    beforeEach(() => {
        pinia = createTestingPinia({
            createSpy: vi.fn,
            stubActions: false,
        })
        servicesStore = useServicesStore()
        servicesStore.services = [
            { id: 's1', name: 'Service 1', endpoints: [] },
            { id: 's2', name: 'Service 2', endpoints: [] }
        ]
        vi.clearAllMocks()
    })

    const mountDialog = (props: any = { open: true }) => {
        return mount(ImportCurlDialog, {
            props,
            global: {
                plugins: [pinia],
                stubs: {
                    DialogPortal: { template: '<div><slot /></div>' },
                    Select: {
                        props: ['modelValue'],
                        template: '<div class="select-stub"><slot /></div>'
                    },
                    SelectTrigger: { template: '<div class="select-trigger"><slot /></div>' },
                    SelectValue: { template: '<span></span>' },
                    SelectContent: { template: '<div><slot /></div>' },
                    SelectItem: { template: '<div><slot /></div>' },
                    Textarea: {
                        props: ['modelValue'],
                        template: '<textarea :value="modelValue" @input="$emit(\'update:modelValue\', $event.target.value)"></textarea>'
                    },
                    'transition': false
                }
            }
        })
    }

    it('renders correctly when open', async () => {
        const wrapper = mountDialog({ open: true })
        await wrapper.vm.$nextTick()

        expect(wrapper.text()).toContain('Import from cURL')
        expect(wrapper.find('textarea').exists()).toBe(true)
    })

    it('pre-selects service if serviceId prop is provided', async () => {
        const wrapper = mountDialog({ open: true, serviceId: 's2' })
        await wrapper.vm.$nextTick()

        const vm = wrapper.vm as any
        expect(vm.selectedServiceId).toBe('s2')
    })

    it('disables import button when inputs are missing', async () => {
        const wrapper = mountDialog({ open: true })
        await wrapper.vm.$nextTick()

        const importButton = wrapper.findAll('button').find(b => b.text().includes('Import Endpoint'))
        expect(importButton?.attributes('disabled')).toBeDefined()
    })

    it('calls store importCurl and emits complete on successful import', async () => {
        const wrapper = mountDialog({ open: true, serviceId: 's1' })
        await wrapper.vm.$nextTick()

        const textarea = wrapper.find('textarea')
        await textarea.setValue('curl https://api.example.com')

        vi.mocked(servicesStore.importCurl).mockResolvedValue({ id: 's1', endpoints: [{ id: 'e1' }] })

        const importButton = wrapper.findAll('button').find(b => b.text().includes('Import Endpoint'))
        // Ensure button is not disabled
        expect(importButton?.attributes('disabled')).toBeUndefined()

        await importButton?.trigger('click')

        expect(servicesStore.importCurl).toHaveBeenCalledWith('s1', 'curl https://api.example.com')
        expect(wrapper.emitted('import-complete')).toBeDefined()
        expect(wrapper.emitted('update:open')?.[0][0]).toBe(false)
        expect(toast.success).toHaveBeenCalled()
    })

    it('shows error toast on failed import', async () => {
        const wrapper = mountDialog({ open: true, serviceId: 's1' })
        await wrapper.vm.$nextTick()

        const textarea = wrapper.find('textarea')
        await textarea.setValue('invalid')

        vi.mocked(servicesStore.importCurl).mockRejectedValue(new Error('Syntax Error'))

        const importButton = wrapper.findAll('button').find(b => b.text().includes('Import Endpoint'))
        await importButton?.trigger('click')

        expect(toast.error).toHaveBeenCalled()
    })
})
