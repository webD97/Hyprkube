import { queryOptions } from "@tanstack/react-query";
import listResourcePresentations from "../../api/listResourcePresentations";
import { KubeContextSource } from "../../hooks/useContextDiscovery";
import { Gvk } from "../../model/k8s";

export default function listResourcePresentationsQuery(contextSource: KubeContextSource, gvk: Gvk) {
    return queryOptions({
        queryKey: ['listResourcePresentations', contextSource, gvk],
        queryFn: () => listResourcePresentations(contextSource, gvk),
    });
}