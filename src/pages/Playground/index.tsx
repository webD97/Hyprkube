import { useSuspenseQuery } from "@tanstack/react-query";
import { Button } from "antd";
import ResourceContextMenu from "../../components/ResourceContextMenu";
import discoverContextsQuery from "../../queries/discoverContexts";

export const Playground: React.FC = () => {
    const contextSources = useSuspenseQuery({
        ...discoverContextsQuery(),
    });

    const source = Object.keys(contextSources.data)[0];
    const contextSource = contextSources.data[source]?.contexts?.[0];

    return (
        <div>
            <h1>Development playground</h1>

            <ResourceContextMenu
                contextSource={contextSource}
                gvk={{ group: "", version: "v1", kind: "Pod" }}
                namespace="monitoring-system"
                name="alertmanager-kube-prometheus-stack-alertmanager-0"
            >
                <Button>alertmanager-kube-prometheus-stack-alertmanager-0</Button>
            </ResourceContextMenu>

            <ResourceContextMenu
                contextSource={contextSource}
                gvk={{ group: "", version: "v1", kind: "Pod" }}
                namespace="monitoring-system"
                name="prometheus-kube-prometheus-stack-prometheus-0"
            >
                <Button>prometheus-kube-prometheus-stack-prometheus-0</Button>
            </ResourceContextMenu>
        </div >
    );
};