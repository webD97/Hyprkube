#{
    name: "Deployments (default)",
    matchApiVersion: "apps/v1",
    matchKind: "Deployment",
    columns: [
        #{
            title: "Name",
            accessor: |obj| {
                obj.metadata.name
            }
        },
        #{
            title: "Namespace",
            accessor: |obj| {
                obj.metadata.namespace
            }
        },
        #{
            title: "Pods",
            accessor: |obj| {
                let desired = obj.status?.replicas;
                let actual = desired - (obj.status?.unavailableReplicas ?? 0);

                `${actual}/${desired}`
            }
        },
        #{
            title: "Status",
            accessor: |obj| {
                let available = obj.status
                    ?.conditions
                    ?.find(|condition| {
                        condition.type == "Available"
                    })
                    ?.status == "True";
                
                if available {
                    return ColoredString("Available", "lime");
                }

                ColoredString("Unavailable", "crimson")
            }
        },
        #{
            title: "Age",
            accessor: |obj| {
                obj.metadata.creationTimestamp
            }
        },
    ]
}
