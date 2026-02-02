import { describe, it, expect } from 'vitest'

describe('Param/Header Transformation Logic', () => {
    it('should preserve enabled flag when transforming params', () => {
        // This is the FIXED transformation logic (now used in ServicesView and CollectionsView)
        const transform = (params: any[]) =>
            params.filter((p: any) => p.name).map((p: any) => ({
                name: p.name,
                value: p.value,
                enabled: p.enabled ?? true  // Preserve enabled, default to true
            }))

        const inputParams = [
            { name: 'id', value: '123', enabled: true },
            { name: 'filter', value: 'active', enabled: false },
            { name: 'sort', value: 'desc', enabled: true }
        ]

        const result = transform(inputParams)

        expect(result).toEqual([
            { name: 'id', value: '123', enabled: true },
            { name: 'filter', value: 'active', enabled: false },
            { name: 'sort', value: 'desc', enabled: true }
        ])
    })

    it('should default enabled to true for params without the flag', () => {
        const transform = (params: any[]) =>
            params.filter((p: any) => p.name).map((p: any) => ({
                name: p.name,
                value: p.value,
                enabled: p.enabled ?? true
            }))

        const inputParams = [
            { name: 'id', value: '123' },  // No enabled flag (legacy data)
            { name: 'filter', value: 'active', enabled: false }
        ]

        const result = transform(inputParams)

        expect(result).toEqual([
            { name: 'id', value: '123', enabled: true },  // Defaulted to true
            { name: 'filter', value: 'active', enabled: false }
        ])
    })

    it('should preserve enabled flag when transforming headers', () => {
        const transform = (headers: any[]) =>
            headers.filter((h: any) => h.name).map((h: any) => ({
                name: h.name,
                value: h.value,
                enabled: h.enabled ?? true
            }))

        const inputHeaders = [
            { name: 'Authorization', value: 'Bearer token', enabled: true },
            { name: 'X-Custom', value: 'test', enabled: false }
        ]

        const result = transform(inputHeaders)

        expect(result).toEqual([
            { name: 'Authorization', value: 'Bearer token', enabled: true },
            { name: 'X-Custom', value: 'test', enabled: false }
        ])
    })

    it('should filter out params with empty names', () => {
        const transform = (params: any[]) =>
            params.filter((p: any) => p.name).map((p: any) => ({
                name: p.name,
                value: p.value,
                enabled: p.enabled ?? true
            }))

        const inputParams = [
            { name: 'id', value: '123', enabled: true },
            { name: '', value: 'should-be-filtered', enabled: true },
            { name: 'filter', value: 'active', enabled: false }
        ]

        const result = transform(inputParams)

        expect(result).toEqual([
            { name: 'id', value: '123', enabled: true },
            { name: 'filter', value: 'active', enabled: false }
        ])
    })
})
