import { type Service, type EnvironmentConfig, AuthType } from '@/types'

export class ServiceFactory {
    static createDefault(name: string, directory: string): Service {
        const id = `s-${Date.now()}`
        return {
            id,
            name,
            directory,
            isAuthenticated: false,
            authType: AuthType.BEARER,
            endpoints: [],
            environments: this.createDefaultEnvironments(),
            selectedEnvironment: 'DEV'
        }
    }

    static createDefaultEnvironments(): EnvironmentConfig[] {
        return [
            {
                name: 'DEV',
                isUnsafe: false,
                variables: [
                    { name: 'BASE_URL', value: 'http://localhost:3000' }
                ]
            },
            {
                name: 'STAGE',
                isUnsafe: false,
                variables: [
                    { name: 'BASE_URL', value: 'https://stage.api.com' }
                ]
            },
            {
                name: 'PROD',
                isUnsafe: true,
                variables: [
                    { name: 'BASE_URL', value: 'https://api.com' }
                ]
            }
        ]
    }
}
