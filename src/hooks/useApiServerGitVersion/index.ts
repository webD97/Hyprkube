import { useQuery } from "@tanstack/react-query";
import getApiServerGitVersion from "../../api/getApiServerGitVersion";
import { KubeContextSource } from "../useContextDiscovery";

export default function useApiServerGitVersion(contextSource: KubeContextSource, enabled?: boolean) {
    return useQuery({
        queryKey: ['getApiServerGitVersion', contextSource],
        queryFn: () => getApiServerGitVersion(contextSource),
        retry: false,
        enabled
    });
}