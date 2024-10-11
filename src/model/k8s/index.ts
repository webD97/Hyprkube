import { ObjectMeta } from "kubernetes-types/meta/v1";

export interface KubernetesApiObject {
    metadata?: ObjectMeta
}

export type Gvk = {
    group: string,
    version: string,
    kind: string
};
