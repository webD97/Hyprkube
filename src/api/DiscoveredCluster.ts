export type DiscoveredCluster = {
    clientId: string,
    discovery: DiscoveryResult
};

export type DiscoveredGroup = {
    name: string,
    isCrd: boolean,
    kinds: DiscoveredResource[]
}

export type DiscoveredResource = {
    version: string,
    kind: string,
    views: string[]
}

export type DiscoveryResult = {
    gvks: { [key: string]: DiscoveredGroup },
    crdApigroups: string[],
    builtinApigroups: string[]
}
