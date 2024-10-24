export type Gvk = {
    group: string,
    version: string,
    kind: string
};

export interface NamespaceAndName {
    namespace?: string,
    name?: string
}
