import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { ref } from 'vue'
import { useRequestExecution } from '@/composables/useRequestExecution'
import { useEnvironmentVariables } from '@/composables/useEnvironmentVariables'
import { RequestGateway } from '@/../bindings/xrest/cmd/wails'


// Mock dependencies
vi.mock('@/../bindings/xrest/cmd/wails', () => ({
    RequestGateway: {
        Send: vi.fn()
    }
}))

vi.mock('vue-sonner', () => ({
    toast: {
        error: vi.fn(),
        success: vi.fn()
    }
}))

vi.mock('@/composables/useEnvironmentVariables', () => ({
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
                preflight: {
                    url: '',
                    method: 'GET',
                    headers: [],
                    body: '',
                    triggerRules: []
                },
                body: { type: 'raw', content: '' }
            }

            handleSendRequest(mockTab, false)

            // Dialog should be open
            expect(isUnsafeDialogOpen.value).toBe(true)
            expect(unsafeTabToProceed.value?.id).toBe(mockTab.id)
            expect(RequestGateway.Send).not.toHaveBeenCalled()
        })

        it('should NOT trigger unsafe dialog when environment is safe', async () => {
            mockIsUnsafeEnv.mockReturnValue(false)
            ; (RequestGateway.Send as any).mockResolvedValue({
                statusCode: 200,
                statusText: 'OK',
                timeTaken: 120 * 1000000,
                size: 1024,
                body: '{"success":true}',
                responseHeaders: {}
            })

            const { handleSendRequest } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                preflight: {
                    url: '',
                    method: 'GET',
                    headers: [],
                    body: '',
                    triggerRules: []
                },
                body: { type: 'raw', content: '' }
            }

            await handleSendRequest(mockTab, false)

            // Dialog should remain closed
            expect(isUnsafeDialogOpen.value).toBe(false)

            // Request should be sent immediately
            expect(RequestGateway.Send).toHaveBeenCalledWith(expect.any(Object))
        })

        it('should skip unsafe warning when skipWarning is true', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)
            ; (RequestGateway.Send as any).mockResolvedValue({
                statusCode: 200,
                statusText: 'OK',
                timeTaken: 120 * 1000000,
                size: 1024,
                body: '{"success":true}',
                responseHeaders: {}
            })

            const { handleSendRequest } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                preflight: {
                    url: '',
                    method: 'GET',
                    headers: [],
                    body: '',
                    triggerRules: []
                },
                body: { type: 'raw', content: '' }
            }

            // Run with skipWarning = true
            await handleSendRequest(mockTab, true)

            // Dialog should remain closed
            expect(isUnsafeDialogOpen.value).toBe(false)

            // Request should be sent immediately
            expect(RequestGateway.Send).toHaveBeenCalledWith(expect.any(Object))
        })
    })

    describe('User Actions', () => {
        let mockTab: any

        beforeEach(() => {
            mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                preflight: {
                    url: '',
                    method: 'GET',
                    headers: [],
                    body: '',
                    triggerRules: []
                },
                body: { type: 'raw', content: '' }
            }
        })

        it('should proceed with request when user accepts the risk', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)
            ; (RequestGateway.Send as any).mockResolvedValue({
                statusCode: 200,
                statusText: 'OK',
                timeTaken: 120 * 1000000,
                size: 1024,
                body: '{"success":true}',
                responseHeaders: {}
            })

            const { handleSendRequest, proceedWithUnsafeRequest, unsafeCountdown } = useRequestExecution(isUnsafeDialogOpen)

            // Start request
            handleSendRequest(mockTab, false)
            expect(isUnsafeDialogOpen.value).toBe(true)

            // Fast-forward countdown (5 seconds)
            vi.advanceTimersByTime(5000)
            expect(unsafeCountdown.value).toBe(0)

            // Accept
            await proceedWithUnsafeRequest(handleSendRequest)

            // Dialog should close
            expect(isUnsafeDialogOpen.value).toBe(false)

            // Request should be sent with skipWarning=true
            expect(RequestGateway.Send).toHaveBeenCalledWith(expect.any(Object))
        })

        it('should not allow proceed when countdown is active', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)

            const { handleSendRequest, proceedWithUnsafeRequest, unsafeCountdown } = useRequestExecution(isUnsafeDialogOpen)

            // Start request
            handleSendRequest(mockTab, false)
            expect(isUnsafeDialogOpen.value).toBe(true)

            // Countdown is at 5
            expect(unsafeCountdown.value).toBe(5)

            // Attempt to accept early
            await proceedWithUnsafeRequest(handleSendRequest)

            // Request should not be sent, dialog remains open
            expect(RequestGateway.Send).not.toHaveBeenCalled()
            expect(isUnsafeDialogOpen.value).toBe(true)
        })

        it('should clean up state when user cancels', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)

            const { handleSendRequest, cancelUnsafeRequest, unsafeTabToProceed } = useRequestExecution(isUnsafeDialogOpen)

            // Start request
            handleSendRequest(mockTab, false)
            expect(isUnsafeDialogOpen.value).toBe(true)
            expect(unsafeTabToProceed.value?.id).toBe(mockTab.id)

            // Cancel
            cancelUnsafeRequest()

            // State should be reset
            expect(isUnsafeDialogOpen.value).toBe(false)
            expect(unsafeTabToProceed.value).toBeNull()
            expect(RequestGateway.Send).not.toHaveBeenCalled()
        })
    })

    describe('Request Blocking', () => {
        let mockTab: any

        beforeEach(() => {
            mockTab = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                preflight: {
                    url: '',
                    method: 'GET',
                    headers: [],
                    body: '',
                    triggerRules: []
                },
                body: { type: 'raw', content: '' }
            }
        })

        it('should block request execution until user accepts', async () => {
            mockIsUnsafeEnv.mockReturnValue(true)
            let resolveRequest: any
            const requestPromise = new Promise((resolve) => {
                resolveRequest = resolve
            })

            ; (RequestGateway.Send as any).mockImplementation(() => {
                resolveRequest({
                    statusCode: 200,
                    statusText: 'OK',
                    timeTaken: 120 * 1000000,
                    size: 1024,
                    body: '{"success":true}',
                    responseHeaders: {}
                })
                return requestPromise
            })

            const { handleSendRequest, proceedWithUnsafeRequest } = useRequestExecution(isUnsafeDialogOpen)

            // Trigger request (enters unsafe check)
            handleSendRequest(mockTab, false)
            expect(RequestGateway.Send).not.toHaveBeenCalled()

            // Wait countdown and proceed
            vi.advanceTimersByTime(5000)
            await proceedWithUnsafeRequest(handleSendRequest)

            // Now request should be sent
            expect(RequestGateway.Send).toHaveBeenCalledTimes(1)
        })

        it('should prevent multiple simultaneous requests', async () => {
            mockIsUnsafeEnv.mockReturnValue(false)

            let resolveRequest: any
            const requestPromise = new Promise((resolve) => {
                resolveRequest = resolve
            })

            ; (RequestGateway.Send as any).mockImplementation(() => requestPromise)

            const { handleSendRequest, isSending } = useRequestExecution(isUnsafeDialogOpen)

            // Trigger first request
            const promise1 = handleSendRequest(mockTab, false)
            expect(isSending.value).toBe(true)

            // Trigger second request immediately
            const promise2 = handleSendRequest(mockTab, false)

            // Should only call invoke once
            expect(RequestGateway.Send).toHaveBeenCalledTimes(1)

            // Wait for first request to complete
            resolveRequest({
                statusCode: 200,
                statusText: 'OK',
                timeTaken: 120 * 1000000,
                size: 1024,
                body: '{"success":true}',
                responseHeaders: {}
            })
            await promise1
            await promise2

            expect(isSending.value).toBe(false)
        })
    })

    describe('Error Handling', () => {
        it('should handle request errors gracefully', async () => {
            mockIsUnsafeEnv.mockReturnValue(false)
            ; (RequestGateway.Send as any).mockRejectedValue(new Error('Network error'))

            const { handleSendRequest } = useRequestExecution(isUnsafeDialogOpen)

            const mockTab: any = {
                id: 'tab-1',
                serviceId: 'service-1',
                url: 'https://api.example.com/users',
                method: 'GET',
                params: [],
                headers: [],
                preflight: {
                    url: '',
                    method: 'GET',
                    headers: [],
                    body: '',
                    triggerRules: []
                },
                body: { type: 'raw', content: '' },
                response: {}
            }

            await handleSendRequest(mockTab, false)

            expect(mockTab.response.error).toBe('Error: Network error')
            expect(mockTab.response.status).toBe(0)
            expect(mockTab.response.statusText).toBe('Error')
        })
    })
})
