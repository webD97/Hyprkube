import { ObjectMeta } from "kubernetes-types/meta/v1";

export interface GenericResource {
    apiVersion?: string,
    kind?: string,
    metadata?: ObjectMeta,
}

export type Gvk = {
    group: string,
    version: string,
    kind: string
};
