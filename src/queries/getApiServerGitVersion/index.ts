import { queryOptions } from "@tanstack/react-query";
import getApiServerGitVersion from "../../api/getApiServerGitVersion";
import { KubeContextSource } from "../../hooks/useContextDiscovery";

export default function getApiServerGitVersionQuery(contextSource: KubeContextSource) {
    return queryOptions({
        queryKey: ['getApiServerGitVersion', contextSource],
        queryFn: () => getApiServerGitVersion(contextSource),
    });
}