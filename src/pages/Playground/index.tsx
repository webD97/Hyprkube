import { useQuery, useSuspenseQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "antd";
import discoverContextsQuery from "../../queries/discoverContexts";

export const Playground: React.FC = () => {
    const contextSources = useSuspenseQuery({
        ...discoverContextsQuery(),
    });

    const source = Object.keys(contextSources.data)[0];
    const contextSource = contextSources.data[source]?.contexts?.[0];

    const menu = useQuery({
        queryKey: ['blubb'],
        enabled: !!contextSource,
        queryFn: () => invoke<{ id: string, items: Array<"Separator" | { ActionButton: { title: string, dangerous: boolean, actionRef: string } }> }>("create_resource_menustack", {
            contextSource,
            gvk: { group: "", version: "v1", kind: "Pod" },
            namespace: "monitoring-system",
            name: "alertmanager-kube-prometheus-stack-alertmanager-0"
        }),
        retry: 0
    });

    if (menu.isError) {
        return <p>{JSON.stringify(menu.error)}</p>;
    }

    if (menu.isPending) {
        return <>Loading</>;
    }

    return (
        <div>
            <h1>Development playground</h1>
            {/* <LazyDropdown
                items={items}
                onSubmenuActivated={(key) => {
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unused-vars
                    const { resource: { name } } = decodeItemKey(key as string);

                    return Promise.resolve([]);
                }}
            >
                <Button>I am lazy loaded 😴</Button>
            </LazyDropdown> */}
            {
                menu.data.items.map(item => {
                    if (typeof (item) === "string" && item === "Separator") {
                        return <></>;
                    }
                    else if ("ActionButton" in item) {
                        return (
                            <Button
                                key={item.ActionButton.title}
                                danger={item.ActionButton.dangerous}
                                onClick={() => void invoke("call_menustack_action", { contextSource, menustackId: menu.data.id, actionRef: item.ActionButton.actionRef })}
                            >
                                {item.ActionButton.title}
                            </Button>
                        );
                    }
                })
            }
            <pre>{JSON.stringify(menu.data, undefined, 2)}</pre>
        </div >
    );
};