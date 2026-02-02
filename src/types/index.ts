export type GitStatus = {
    isGit: boolean;
    remoteUrl?: string;
    branch?: string;
    hasUncommittedChanges: boolean;
    hasUnpushedCommits: boolean;
    lastSync?: number;
}

export type Service = {
    id: string;
    name: string;
    environments: EnvironmentConfig[];
    isAuthenticated: boolean;
    authType?: AuthType;
    endpoints: Endpoint[];
    directory: string;
    selectedEnvironment?: string;
    gitUrl?: string;
}

export type NameValue = {
    name: string;
    value: string;
}

export type Variable = {
    name: string;
    value: string;
    enabled?: boolean;
    secretKey?: string;
}
export type Param = NameValue;
export type Header = NameValue;

export type EnvironmentConfig = {
    name: string;
    isUnsafe: boolean;
    variables: Variable[];
}

export enum AuthType {
    BEARER = 'bearer',
    API_KEY = 'apikey',
    BASIC = 'basic',
    NONE = 'none'
}

export type EndpointMetadata = {
    version: string;
    lastUpdated: number;
}

export type PreflightConfig = {
    enabled: boolean;
    method: string;
    url: string;
    body: string;
    headers: Header[];
    cacheToken: boolean;
    cacheDuration: string;
    cacheDurationKey: string;
    cacheDurationUnit: string;
    tokenKey: string;
    tokenHeader: string;
}

export type RequestConfig = {
    method: string;
    url: string;
    authenticated: boolean;
    authType: string;
    params: Param[];
    headers: Header[];
    body: string;
    preflight: PreflightConfig;
}

export type EndpointVersion = {
    version: number;
    config: RequestConfig;
    lastUpdated: number;
}

export type Endpoint = {
    id: string;
    serviceId: string;
    name: string;
    method: string;
    url: string;
    authenticated: boolean;
    authType: string;
    metadata: EndpointMetadata;
    params: Param[];
    headers: Header[];
    body: string;
    preflight: PreflightConfig;
    lastVersion?: number;
    versions?: EndpointVersion[];
}

// Response is a protected type in TS
export type QResponse = {
    status: number;
    statusText: string;
    headers: Header[];
    body: string;
    error?: string;
    timeElapsed: number;
    size: number;
}