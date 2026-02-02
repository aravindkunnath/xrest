import { type Service, type EnvironmentConfig, AuthType } from '@/types'

export class CollectionFactory {
    static createDefault(name: string): Service {
        const id = `c-${Date.now()}`
        return {
            id,
            name,
            directory: '', // Collections are not file-based
            isAuthenticated: false,
            authType: AuthType.NONE,
            endpoints: [],
            environments: this.createDefaultEnvironments(),
            selectedEnvironment: 'GLOBAL'
        }
    }

    private static createDefaultEnvironments(): EnvironmentConfig[] {
        return [
            {
                name: 'GLOBAL',
                isUnsafe: false,
                variables: [
                    { name: 'BASE_URL', value: 'https://api.example.com' }
                ]
            }
        ]
    }
}
