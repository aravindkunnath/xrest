import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { ref } from 'vue'
import { useRequestExecution } from '../useRequestExecution'
import { useEnvironmentVariables } from '../useEnvironmentVariables'
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'

// Mock dependencies
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn()
}))

vi.mock('vue-sonner', () => ({
    toast: {
        error: vi.fn(),
        success: vi.fn()
    }
}))

vi.mock('../useEnvironmentVariables', () => ({
    useEnvironmentVariables: vi.fn()
}))

describe('useRequestExecution - Unsafe Environment Flow', () => {
    let isUnsafeDialogOpen: any
    let mockGetTabVariables: any
    let mockIsUnsafeEnv: any

    beforeEach(() => {
        vi.clearAllMocks()
        vi.useFakeTimers()

        isUnsafeDialogOpen = ref(false)
        mockGetTabVariables = vi.fn().mockReturnValue({})
        mockIsUnsafeEnv = vi.fn().mockReturnValue(false)

            // Setup mock for useEnvironmentVariables
            ; (useEnvironmentVariables as any).mockReturnValue({
                getTabVariables: mockGetTabVariables,
                isUnsafeEnv: mockIsUnsafeEnv
            })
    })

    afterEach(() => {
        vi.useRealTimers()
    })

    describe('Unsafe Environment Detection', () => {
        it('should trigger unsafe dialog when environment is unsafe', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)

            const { handleSendRequest, unsafeTabToProceed } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {}
            }

            await handleSendRequest(mockTab, false)

            // Dialog should be opened
            expect(isUnsafeDialogOpen.value).toBe(true)

            // Tab should be stored for later execution
            expect(unsafeTabToProceed.value).toStrictEqual(mockTab)

            // Request should NOT be sent yet
            expect(invoke).not.toHaveBeenCalled()
        })

        it('should NOT trigger unsafe dialog when environment is safe', async () => {
            mockIsUnsafeEnv.mockReturnValue(false)
                ; (invoke as any).mockResolvedValue({
                    status: 200,
                    statusText: 'OK',
                    timeElapsed: 100,
                    size: 1024,
                    body: '{"success": true}',
                    headers: []
                })

            const { handleSendRequest } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {},
                response: {}
            }

            await handleSendRequest(mockTab, false)

            // Dialog should NOT be opened
            expect(isUnsafeDialogOpen.value).toBe(false)

            // Request should be sent immediately
            expect(invoke).toHaveBeenCalledWith('send_request', expect.any(Object))
        })

        it('should skip unsafe warning when skipWarning is true', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)
                ; (invoke as any).mockResolvedValue({
                    status: 200,
                    statusText: 'OK',
                    timeElapsed: 100,
                    size: 1024,
                    body: '{"success": true}',
                    headers: []
                })

            const { handleSendRequest } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {},
                response: {}
            }

            // Skip warning by passing true as second parameter
            await handleSendRequest(mockTab, true)

            // Dialog should NOT be opened
            expect(isUnsafeDialogOpen.value).toBe(false)

            // Request should be sent immediately
            expect(invoke).toHaveBeenCalledWith('send_request', expect.any(Object))
        })
    })

    describe('Countdown Timer', () => {
        it('should start countdown at 10 seconds when unsafe dialog opens', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)

            const { handleSendRequest, unsafeCountdown } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {}
            }

            await handleSendRequest(mockTab, false)

            expect(unsafeCountdown.value).toBe(10)
        })

        it('should decrement countdown every second', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)

            const { handleSendRequest, unsafeCountdown } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {}
            }

            await handleSendRequest(mockTab, false)

            expect(unsafeCountdown.value).toBe(10)

            // Advance timer by 1 second
            vi.advanceTimersByTime(1000)
            expect(unsafeCountdown.value).toBe(9)

            // Advance timer by 3 more seconds
            vi.advanceTimersByTime(3000)
            expect(unsafeCountdown.value).toBe(6)
        })

        it('should auto-cancel request when countdown reaches 0', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)

            const { handleSendRequest, unsafeTabToProceed } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {}
            }

            await handleSendRequest(mockTab, false)

            expect(isUnsafeDialogOpen.value).toBe(true)
            expect(unsafeTabToProceed.value).toStrictEqual(mockTab)

            // Advance timer by 10 seconds to trigger auto-cancel
            vi.advanceTimersByTime(10000)

            // Dialog should be closed
            expect(isUnsafeDialogOpen.value).toBe(false)

            // Tab should be cleared
            expect(unsafeTabToProceed.value).toBe(null)

            // Request should NOT be sent
            expect(invoke).not.toHaveBeenCalled()
        })
    })

    describe('User Actions', () => {
        it('should proceed with request when user accepts the risk', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)
                ; (invoke as any).mockResolvedValue({
                    status: 200,
                    statusText: 'OK',
                    timeElapsed: 100,
                    size: 1024,
                    body: '{"success": true}',
                    headers: []
                })

            const { handleSendRequest, proceedWithUnsafeRequest, unsafeTabToProceed } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {},
                response: {}
            }

            // Trigger unsafe dialog
            await handleSendRequest(mockTab, false)

            expect(isUnsafeDialogOpen.value).toBe(true)
            expect(unsafeTabToProceed.value).toStrictEqual(mockTab)
            expect(invoke).not.toHaveBeenCalled()

            // User accepts the risk
            proceedWithUnsafeRequest(handleSendRequest)

            // Wait for async operations
            await vi.runAllTimersAsync()

            // Dialog should be closed
            expect(isUnsafeDialogOpen.value).toBe(false)

            // Tab should be cleared
            expect(unsafeTabToProceed.value).toBe(null)

            // Request should be sent with skipWarning=true
            expect(invoke).toHaveBeenCalledWith('send_request', expect.any(Object))
        })

        it('should cancel request when user clicks cancel', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)

            const { handleSendRequest, cancelUnsafeRequest, unsafeTabToProceed } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {}
            }

            // Trigger unsafe dialog
            await handleSendRequest(mockTab, false)

            expect(isUnsafeDialogOpen.value).toBe(true)
            expect(unsafeTabToProceed.value).toStrictEqual(mockTab)

            // User cancels
            cancelUnsafeRequest()

            // Dialog should be closed
            expect(isUnsafeDialogOpen.value).toBe(false)

            // Tab should be cleared
            expect(unsafeTabToProceed.value).toBe(null)

            // Request should NOT be sent
            expect(invoke).not.toHaveBeenCalled()
        })

        it('should stop countdown timer when user accepts', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)
                ; (invoke as any).mockResolvedValue({
                    status: 200,
                    statusText: 'OK',
                    timeElapsed: 100,
                    size: 1024,
                    body: '{"success": true}',
                    headers: []
                })

            const { handleSendRequest, proceedWithUnsafeRequest, unsafeCountdown } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {},
                response: {}
            }

            await handleSendRequest(mockTab, false)

            // Advance timer by 3 seconds
            vi.advanceTimersByTime(3000)
            expect(unsafeCountdown.value).toBe(7)

            // User accepts
            proceedWithUnsafeRequest(handleSendRequest)
            await vi.runAllTimersAsync()

            // Advance timer further - countdown should not continue
            vi.advanceTimersByTime(5000)
            expect(unsafeCountdown.value).toBe(7) // Should stay at 7
        })

        it('should stop countdown timer when user cancels', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)

            const { handleSendRequest, cancelUnsafeRequest, unsafeCountdown } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {}
            }

            await handleSendRequest(mockTab, false)

            // Advance timer by 4 seconds
            vi.advanceTimersByTime(4000)
            expect(unsafeCountdown.value).toBe(6)

            // User cancels
            cancelUnsafeRequest()

            // Advance timer further - countdown should not continue
            vi.advanceTimersByTime(10000)
            expect(unsafeCountdown.value).toBe(6) // Should stay at 6
        })
    })

    describe('Request Blocking', () => {
        it('should block request execution until user accepts', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)
                ; (invoke as any).mockResolvedValue({
                    status: 200,
                    statusText: 'OK',
                    timeElapsed: 100,
                    size: 1024,
                    body: '{"success": true}',
                    headers: []
                })

            const { handleSendRequest, proceedWithUnsafeRequest } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {},
                response: {}
            }

            // First call - should trigger dialog
            await handleSendRequest(mockTab, false)
            expect(invoke).not.toHaveBeenCalled()

            // Advance time - request should still not be sent
            vi.advanceTimersByTime(5000)
            expect(invoke).not.toHaveBeenCalled()

            // User accepts
            proceedWithUnsafeRequest(handleSendRequest)
            await vi.runAllTimersAsync()

            // Now request should be sent
            expect(invoke).toHaveBeenCalledTimes(1)
        })

        it('should prevent multiple simultaneous requests', async () => {
            mockIsUnsafeEnv.mockReturnValue(false)
                ; (invoke as any).mockResolvedValue({
                    status: 200,
                    statusText: 'OK',
                    timeElapsed: 100,
                    size: 1024,
                    body: '{"success": true}',
                    headers: []
                })

            const { handleSendRequest, isSending } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {},
                response: {}
            }

            // Start first request
            const promise1 = handleSendRequest(mockTab, false)

            // Try to start second request while first is in progress
            await handleSendRequest(mockTab, false)

            // Should only call invoke once
            expect(invoke).toHaveBeenCalledTimes(1)

            // Wait for first request to complete
            await promise1
            await vi.runAllTimersAsync()

            expect(isSending.value).toBe(false)
        })
    })

    describe('Error Handling', () => {
        it('should show error toast when URL is missing', async () => {
            mockIsUnsafeEnv.mockReturnValue(false)

            const { handleSendRequest } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: '',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {},
                response: {
                    status: 0,
                    statusText: '',
                    time: '',
                    size: '',
                    body: '',
                    headers: [],
                    error: ''
                }
            }

            await handleSendRequest(mockTab, false)

            expect(toast.error).toHaveBeenCalledWith('URL is required')
            expect(invoke).not.toHaveBeenCalled()
        })

        it('should handle request errors gracefully', async () => {
            mockIsUnsafeEnv.mockReturnValue(false)
                ; (invoke as any).mockRejectedValue(new Error('Network error'))

            const { handleSendRequest } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                body: { content: '' },
                auth: {},
                preflight: {},
                response: {
                    status: 0,
                    statusText: '',
                    time: '',
                    size: '',
                    body: '',
                    headers: [],
                    error: ''
                }
            }

            await handleSendRequest(mockTab, false)

            expect(mockTab.response.error).toBe('Error: Network error')
            expect(mockTab.response.status).toBe(0)
            expect(mockTab.response.statusText).toBe('Error')
        })
    })
})
